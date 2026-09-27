"""
Export the Transkun V2 piano transcription model (https://github.com/Yujia-Yan/Transkun)
to ONNX, ready to be converted to `.rten` with `rten-convert`.

The exported graph has two independent parts, rten only evaluates the nodes
needed for the requested outputs, so each part can be run on its own:

  frames [1, 2, T, 4096]  -> score [P, T, T], ctx [P, T, D]
  attr   [N, 3 * D]       -> velocity [N, 128], of [N, 4]

- `frames` are stereo audio frames (44.1kHz, hop 1024, window 4096),
- `score[p, end, begin]` is the semi-CRF interval score for event type `p`,
- `ctx` are per frame features used to compute interval attributes,
- `attr` is `[ctx[begin], ctx[end], ctx[begin] * ctx[end]]` for each decoded interval,
- `velocity` are velocity logits, `of` are refined onset/offset logits + presence logits.

Usage:
    python transkun_export.py --transkun /path/to/Transkun --output transkun.onnx
    rten-convert transkun.onnx transkun.rten
"""

import argparse
import math
import sys

import torch
import torch.nn as nn
import torch.nn.functional as F


def attention(q, k, v):
    # Equivalent of `F.scaled_dot_product_attention`, the ONNX exporter
    # only supports it with 4D inputs, while Transkun uses 5D ones.
    attn = (q @ k.transpose(-1, -2)) / math.sqrt(q.shape[-1])
    return attn.softmax(dim=-1) @ v


def mha_forward(self, query, key=None, value=None):
    if key is None:
        key = query
    if value is None:
        value = key

    q = query @ self.q_proj_weight
    k = key @ self.k_proj_weight
    v = value @ self.v_proj_weight

    # split into heads
    q = q.unflatten(-1, (self.num_heads, self.head_dim)).transpose(-2, -3)
    k = k.unflatten(-1, (self.num_heads, self.head_dim)).transpose(-2, -3)
    v = v.unflatten(-1, (self.num_heads, self.head_dim)).transpose(-2, -3)

    fetched = attention(q, k, v)
    fetched = fetched.transpose(-2, -3).flatten(-2, -1)

    return self.out_proj(fetched)


class TransKunExport(nn.Module):
    def __init__(self, model):
        super().__init__()
        self.model = model

        fe = model.framewiseFeatureExtractor
        spec = fe.spectrogramExtractor

        # Learnable windows are constant at inference time, bake them in
        with torch.no_grad():
            wins = [spec.win.unsqueeze(0)]
            if spec.nExtraWins > 0:
                wins.append(spec.winGen.get().t())
            wins = torch.cat(wins, dim=0)

        self.register_buffer("wins", wins)
        self.register_buffer("freq2mels", fe.freq2mels.clone())
        self.register_buffer(
            "targetMIDIPitch", torch.tensor(model.targetMIDIPitch, dtype=torch.float)
        )
        self.eps = fe.eps
        self.windowSize = model.windowSize

    def features(self, frames):
        # frames: [1, nAudioChannel, nFrame, windowSize]

        # gain normalization
        mean = frames.mean(dim=[1, 2, 3], keepdim=True)
        std = frames.std(dim=[1, 2, 3], keepdim=True)
        frames = (frames - mean) / (std + 1e-8)

        # [1, nAudioChannel, nFrame, nWin, windowSize]
        x = frames.unsqueeze(-2) * self.wins

        # |rfft(x, norm="ortho")|^2
        spec = torch.view_as_real(torch.fft.rfft(x))
        power = spec.pow(2).sum(-1) / self.windowSize

        # to mono
        power = power.mean(dim=1)
        # [1, nFrame, nWin, nFreq]

        mel = power @ self.freq2mels
        mel = ((mel + self.eps).log() - math.log(self.eps)) / (-math.log(self.eps))

        # [1, nFrame, nMel, nWin]
        return mel.transpose(-1, -2)

    def forward(self, frames, attr):
        model = self.model

        features = self.features(frames)
        ctx = model.backbone(features, outputIndices=self.targetMIDIPitch)
        # ctx: [1, P, T, D]

        scorer = model.scorer
        q, k, diag = scorer.map(ctx).split(
            [scorer.size * scorer.expansionFactor, scorer.size * scorer.expansionFactor, 1],
            dim=-1,
        )
        q = q / math.sqrt(q.shape[-1])

        # [1, P, T_end, T_begin]
        score = q @ k.transpose(-1, -2)

        t = torch.arange(score.shape[-1], dtype=torch.float)
        len_eb = (t.unsqueeze(-1) - t.unsqueeze(0)).abs()
        score = score * len_eb + torch.diag_embed(diag.squeeze(-1))

        velocity = model.velocityPredictor(attr)
        of = model.refinedOFPredictor(attr)

        return score[0], ctx[0], velocity, of


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--transkun", required=True, help="path to the Transkun repo")
    parser.add_argument("--weight", help="default: <transkun>/transkun/pretrained/2.0.pt")
    parser.add_argument("--conf", help="default: <transkun>/transkun/pretrained/2.0.conf")
    parser.add_argument("--output", default="transkun.onnx")
    args = parser.parse_args()

    sys.path.insert(0, args.transkun)
    import moduleconf
    from transkun import LayersTransformer

    LayersTransformer.MultiHeadAttentionKernel.forward = mha_forward

    weight = args.weight or f"{args.transkun}/transkun/pretrained/2.0.pt"
    conf = args.conf or f"{args.transkun}/transkun/pretrained/2.0.conf"

    confManager = moduleconf.parseFromFile(conf)
    TransKun = confManager["Model"].module.TransKun
    conf = confManager["Model"].config

    model = TransKun(conf=conf)
    checkpoint = torch.load(weight, map_location="cpu")
    state_dict = checkpoint.get("best_state_dict", checkpoint.get("state_dict"))
    model.load_state_dict(state_dict, strict=False)
    model.eval()
    model.backbone.useGradientCheckpoint = False

    export = TransKunExport(model).eval()

    segment_size = math.ceil(model.segmentSizeInSecond * model.fs)
    n_frames = math.ceil(segment_size / model.hopSize) + 1
    d = conf.baseSize * conf.scoringExpansionFactor

    frames = torch.randn(1, 2, n_frames, model.windowSize)
    attr = torch.randn(4, 3 * d)

    with torch.no_grad():
        torch.onnx.export(
            export,
            (frames, attr),
            args.output,
            input_names=["frames", "attr"],
            output_names=["score", "ctx", "velocity", "of"],
            dynamic_shapes={"frames": None, "attr": {0: torch.export.Dim("n_intervals")}},
            dynamo=True,
            external_data=False,
        )

    print(f"Exported: {args.output}")
    print(f"  segment frames: {n_frames}")
    print(f"  event types: {model.targetMIDIPitch}")


if __name__ == "__main__":
    main()
