struct Input
{
    float2 position : TEXCOORD0;
};

struct Output
{
    float2 uv : TEXCOORD0;
    float4 gl_Position : SV_Position;
};

Output main(Input input)
{
    Output output;

    output.gl_Position = float4(input.position,0.0f,1.0f);
    output.uv = (input.position + 1.0f.xx) / 2.0f.xx;

    return output;
}