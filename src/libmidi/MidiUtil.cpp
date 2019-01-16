// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <netinet/in.h>

#include "MidiUtil.h"

using namespace std;

unsigned long BigToSystem32(unsigned long x) {
  return ntohl(x);
}

unsigned short BigToSystem16(unsigned short x) {
  return ntohs(x);
}

unsigned long parse_variable_length(istream &in) {
  register unsigned long value = in.get();

  if (in.good() && (value & 0x80) ) {
    value &= 0x7F;

    register unsigned long c;
    do {
      c = in.get();
      value = (value << 7) + (c & 0x7F);
    } while (in.good() && (c & 0x80) );
  }

  return(value);
}

string MidiError::GetErrorDescription() const {
  switch (m_error) {
  case MidiError_UnknownHeaderType:
    return "Found an unknown header type.\n\nThis probably isn't a valid MIDI file.";
  case MidiError_BadFilename:
    return "Could not open file for input. Check that file exists.";
  case MidiError_NoHeader:
    return "No MIDI header could be read.  File too short.";
  case MidiError_BadHeaderSize:
    return "Incorrect header size.";
  case MidiError_Type2MidiNotSupported:
    return "Type 2 MIDI is not supported.";
  case MidiError_BadType0Midi:
    return "Type 0 MIDI should only have 1 track.";
  case MidiError_SMTPETimingNotImplemented:
    return "MIDI using SMTP time division is not implemented.";

  case MidiError_BadTrackHeaderType:
    return "Found an unknown track header type.";
  case MidiError_TrackHeaderTooShort:
    return "File terminated before reading track header.";
  case MidiError_TrackTooShort:
    return "Data stream too short to read entire track.";
  case MidiError_BadTrackEnd:
    return "MIDI track did not end with End-Of-Track event.";

  case MidiError_EventTooShort:
    return "Data stream ended before reported end of MIDI event.";
  case MidiError_UnknownEventType:
    return "Found an unknown MIDI Event Type.";
  case MidiError_UnknownMetaEventType:
    return "Found an unknown MIDI Meta Event Type.";

  case MidiError_MM_NoDevice:
    return "Could not open the specified MIDI device.";
  case MidiError_MM_NotEnabled:
    return "MIDI device failed enable.";
  case MidiError_MM_AlreadyAllocated:
    return "The specified MIDI device is already in use.";
  case MidiError_MM_BadDeviceID:
    return "The MIDI device ID specified is out of range.";
  case MidiError_MM_InvalidParameter:
    return "An invalid parameter was used with the MIDI device.";
  case MidiError_MM_NoDriver:
    return "The specified MIDI driver is not installed.";
  case MidiError_MM_NoMemory:
    return "Cannot allocate or lock memory for MIDI device.";
  case MidiError_MM_Unknown:
    return "An unknown MIDI I/O error has occurred.";

  case MidiError_NoInputAvailable:
    return "Attempted to read MIDI event from an empty input buffer.";
  case MidiError_MetaEventOnInput:
    return "MIDI Input device sent a Meta Event.";

  case MidiError_InputError:
    return "MIDI input driver reported an error.";
  case MidiError_InvalidInputErrorBehavior:
    return "Invalid InputError value.  Choices are 'report', 'ignore', and 'use'.";

  case MidiError_RequestedTempoFromNonTempoEvent:
    return "Tempo data was requested from a non-tempo MIDI event.";

  default:
    return STRING("Unknown MidiError Code (" << m_error << ").");
  }
}

string GetMidiEventTypeDescription(MidiEventType type) {
  switch (type) {
  case MidiEventType_Meta:             return "Meta";
  case MidiEventType_SysEx:            return "System Exclusive";

  case MidiEventType_NoteOff:          return "Note-Off";
  case MidiEventType_NoteOn:           return "Note-On";
  case MidiEventType_Aftertouch:       return "Aftertouch";
  case MidiEventType_Controller:       return "Controller";
  case MidiEventType_ProgramChange:    return "Program Change";
  case MidiEventType_ChannelPressure:  return "Channel Pressure";
  case MidiEventType_PitchWheel:       return "Pitch Wheel";

  case MidiEventType_Unknown:          return "Unknown";
  default:                             return "BAD EVENT TYPE";
  }
}

string GetMidiMetaEventTypeDescription(MidiMetaEventType type) {
  switch (type) {
  case MidiMetaEvent_SequenceNumber:   return "Sequence Number";

  case MidiMetaEvent_Text:             return "Text";
  case MidiMetaEvent_Copyright:        return "Copyright";
  case MidiMetaEvent_TrackName:        return "Track Name";
  case MidiMetaEvent_Instrument:       return "Instrument";
  case MidiMetaEvent_Lyric:            return "Lyric";
  case MidiMetaEvent_Marker:           return "Marker";
  case MidiMetaEvent_Cue:              return "Cue Point";
  case MidiMetaEvent_PatchName:        return "Patch Name";
  case MidiMetaEvent_DeviceName:       return "Device Name";

  case MidiMetaEvent_EndOfTrack:       return "End Of Track";
  case MidiMetaEvent_TempoChange:      return "Tempo Change";
  case MidiMetaEvent_SMPTEOffset:      return "SMPTE Offset";
  case MidiMetaEvent_TimeSignature:    return "Time Signature";
  case MidiMetaEvent_KeySignature:     return "Key Signature";

  case MidiMetaEvent_Proprietary:      return "Proprietary";

  case MidiMetaEvent_ChannelPrefix:    return "(Deprecated) Channel Prefix";
  case MidiMetaEvent_MidiPort:         return "(Deprecated) MIDI Port";

  case MidiMetaEvent_Unknown:          return "Unknown Meta Event Type";
  default:                             return "BAD META EVENT TYPE";
  }
}

string const InstrumentNames[InstrumentCount] = {
  "Acoustic Grand Piano",
  "Bright Acoustic Piano",
  "Electric Grand Piano",
  "Honky-tonk Piano",
  "Electric Piano 1",
  "Electric Piano 2",
  "Harpsichord",
  "Clavi",
  "Celesta",
  "Glockenspiel",
  "Music Box",
  "Vibraphone",
  "Marimba",
  "Xylophone",
  "Tubular Bells",
  "Dulcimer",
  "Drawbar Organ",
  "Percussive Organ",
  "Rock Organ",
  "Church Organ",
  "Reed Organ",
  "Accordion",
  "Harmonica",
  "Tango Accordion",
  "Acoustic Guitar (nylon)",
  "Acoustic Guitar (steel)",
  "Electric Guitar (jazz)",
  "Electric Guitar (clean)",
  "Electric Guitar (muted)",
  "Overdriven Guitar",
  "Distortion Guitar",
  "Guitar harmonics",
  "Acoustic Bass",
  "Electric Bass (finger)",
  "Electric Bass (pick)",
  "Fretless Bass",
  "Slap Bass 1",
  "Slap Bass 2",
  "Synth Bass 1",
  "Synth Bass 2",
  "Violin",
  "Viola",
  "Cello",
  "Contrabass",
  "Tremolo Strings",
  "Pizzicato Strings",
  "Orchestral Harp",
  "Timpani",
  "String Ensemble 1",
  "String Ensemble 2",
  "SynthStrings 1",
  "SynthStrings 2",
  "Choir Aahs",
  "Voice Oohs",
  "Synth Voice",
  "Orchestra Hit",
  "Trumpet",
  "Trombone",
  "Tuba",
  "Muted Trumpet",
  "French Horn",
  "Brass Section",
  "SynthBrass 1",
  "SynthBrass 2",
  "Soprano Sax",
  "Alto Sax",
  "Tenor Sax",
  "Baritone Sax",
  "Oboe",
  "English Horn",
  "Bassoon",
  "Clarinet",
  "Piccolo",
  "Flute",
  "Recorder",
  "Pan Flute",
  "Blown Bottle",
  "Shakuhachi",
  "Whistle",
  "Ocarina",
  "Lead 1 (square)",
  "Lead 2 (sawtooth)",
  "Lead 3 (calliope)",
  "Lead 4 (chiff)",
  "Lead 5 (charang)",
  "Lead 6 (voice)",
  "Lead 7 (fifths)",
  "Lead 8 (bass + lead)",
  "Pad 1 (new age)",
  "Pad 2 (warm)",
  "Pad 3 (polysynth)",
  "Pad 4 (choir)",
  "Pad 5 (bowed)",
  "Pad 6 (metallic)",
  "Pad 7 (halo)",
  "Pad 8 (sweep)",
  "FX 1 (rain)",
  "FX 2 (soundtrack)",
  "FX 3 (crystal)",
  "FX 4 (atmosphere)",
  "FX 5 (brightness)",
  "FX 6 (goblins)",
  "FX 7 (echoes)",
  "FX 8 (sci-fi)",
  "Sitar",
  "Banjo",
  "Shamisen",
  "Koto",
  "Kalimba",
  "Bag pipe",
  "Fiddle",
  "Shanai",
  "Tinkle Bell",
  "Agogo",
  "Steel Drums",
  "Woodblock",
  "Taiko Drum",
  "Melodic Tom",
  "Synth Drum",
  "Reverse Cymbal",
  "Guitar Fret Noise",
  "Breath Noise",
  "Seashore",
  "Bird Tweet",
  "Telephone Ring",
  "Helicopter",
  "Applause",
  "Gunshot",

  //
  // NOTE: These aren't actually General MIDI instruments!
  //
  "Percussion", // for Tracks that use Channel 10 or 16
  "Various"     // for Tracks that use more than one
};
