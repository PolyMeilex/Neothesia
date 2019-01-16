// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "MidiEvent.h"
#include "MidiUtil.h"
#include "Note.h"

using namespace std;

MidiEvent MidiEvent::ReadFromStream(istream &stream, 
				    unsigned char last_status, 
				    bool contains_delta_pulses) {
  MidiEvent ev;

  if (contains_delta_pulses)
    ev.m_delta_pulses = parse_variable_length(stream);
  else
    ev.m_delta_pulses = 0;

  // MIDI uses a compression mechanism called "running status".
  // Anytime you read a status byte that doesn't have the highest-
  // order bit set, what you actually read is the 1st data byte
  // of a message with the status of the previous message.
  ev.m_status = static_cast<unsigned char>(stream.peek());

  if ((ev.m_status & 0x80) == 0)
    ev.m_status = last_status;

  else
    // It was a status byte after all, just read past it
    // in the stream
    stream.read(reinterpret_cast<char*>(&ev.m_status), sizeof(unsigned char));

  switch (ev.Type()) {
  case MidiEventType_Meta:
    ev.ReadMeta(stream);
    break;

  case MidiEventType_SysEx:
    ev.ReadSysEx(stream);
    break;

  default:
    ev.ReadStandard(stream);
    break;
  }

  return ev;
}

MidiEvent MidiEvent::Build(const MidiEventSimple &simple) {
  MidiEvent ev;

  ev.m_delta_pulses = 0;
  ev.m_status = simple.status;
  ev.m_data1 = simple.byte1;
  ev.m_data2 = simple.byte2;

  if (ev.Type() == MidiEventType_Meta)
    throw MidiError(MidiError_MetaEventOnInput);

  return ev;
}

MidiEvent MidiEvent::NullEvent() {
  MidiEvent ev;

  ev.m_status = 0xFF;
  ev.m_meta_type = MidiMetaEvent_Proprietary;
  ev.m_delta_pulses = 0;

  return ev;
}

void MidiEvent::ReadMeta(istream &stream) {
  stream.read(reinterpret_cast<char*>(&m_meta_type), sizeof(unsigned char));
  unsigned long meta_length = parse_variable_length(stream);

  char *buffer = new char[meta_length + 1];
  buffer[meta_length] = 0;

  stream.read(buffer, meta_length);
  if (stream.fail()) {
    delete[] buffer;
    throw MidiError(MidiError_EventTooShort);
  }

  switch (m_meta_type) {
  case MidiMetaEvent_Text:
  case MidiMetaEvent_Copyright:
  case MidiMetaEvent_TrackName:
  case MidiMetaEvent_Instrument:
  case MidiMetaEvent_Lyric:
  case MidiMetaEvent_Marker:
  case MidiMetaEvent_Cue:
  case MidiMetaEvent_PatchName:
  case MidiMetaEvent_DeviceName:
    m_text = string(buffer, meta_length);
    break;

  case MidiMetaEvent_TempoChange: {
    if (meta_length < 3)
      throw MidiError(MidiError_EventTooShort);

    // We have to convert to unsigned char first for some reason or the
    // conversion gets all wacky and tries to look at more than just its
    // one byte at a time.
    unsigned int b0 = static_cast<unsigned char>(buffer[0]);
    unsigned int b1 = static_cast<unsigned char>(buffer[1]);
    unsigned int b2 = static_cast<unsigned char>(buffer[2]);
    m_tempo_uspqn = (b0 << 16) + (b1 << 8) + b2;

    break;
  }

  case MidiMetaEvent_SequenceNumber:
  case MidiMetaEvent_EndOfTrack:
  case MidiMetaEvent_SMPTEOffset:
  case MidiMetaEvent_TimeSignature:
  case MidiMetaEvent_KeySignature:
  case MidiMetaEvent_Proprietary:
  case MidiMetaEvent_ChannelPrefix:
  case MidiMetaEvent_MidiPort:
    // NOTE: We would have to keep all of this around if we
    // wanted to reproduce 1:1 MIDIs between file Save/Load
    break;

  default:
    // Ignore unknown event
    std::cerr << "Ignore unknown midi event type " << (m_meta_type * 1)
              << " of length " << meta_length << endl;
//  delete[] buffer;
//  throw MidiError(MidiError_UnknownMetaEventType);
  }

  delete[] buffer;
}

void MidiEvent::ReadSysEx(istream &stream) {
  // NOTE: We would have to keep SysEx events around if we
  // wanted to reproduce 1:1 MIDIs between file Save/Load
  unsigned long sys_ex_length = parse_variable_length(stream);

  // Discard
  char *buffer = new char[sys_ex_length];
  stream.read(buffer, sys_ex_length);
  delete[] buffer;
}

void MidiEvent::ReadStandard(istream &stream) {
  switch (Type()) {
  case MidiEventType_NoteOff:
  case MidiEventType_NoteOn:
  case MidiEventType_Aftertouch:
  case MidiEventType_Controller:
  case MidiEventType_PitchWheel:
    stream.read(reinterpret_cast<char*>(&m_data1), sizeof(unsigned char));
    stream.read(reinterpret_cast<char*>(&m_data2), sizeof(unsigned char));
    break;

  case MidiEventType_ProgramChange:
  case MidiEventType_ChannelPressure:
    stream.read(reinterpret_cast<char*>(&m_data1), sizeof(unsigned char));
    m_data2 = 0;
    break;

  default:
    throw MidiError(MidiError_UnknownEventType);
  }
}

bool MidiEvent::GetSimpleEvent(MidiEventSimple *simple) const {
  MidiEventType t = Type();
  if (t == MidiEventType_Meta ||
      t == MidiEventType_SysEx ||
      t == MidiEventType_Unknown)
    return false;

  simple->status = m_status;
  simple->byte1 = m_data1;
  simple->byte2 = m_data2;

  return true;
}

MidiEventType MidiEvent::Type() const {
  if (m_status >  0xEF && m_status < 0xFF)
    return MidiEventType_SysEx;

  if (m_status <  0x80)
    return MidiEventType_Unknown;

  if (m_status == 0xFF)
    return MidiEventType_Meta;

  // The 0x8_ through 0xE_ events contain channel numbers
  // in the lowest 4 bits
  unsigned char status_top = m_status >> 4;

  switch (status_top) {
  case 0x8:
    return MidiEventType_NoteOff;

  case 0x9:
    return MidiEventType_NoteOn;

  case 0xA:
    return MidiEventType_Aftertouch;

  case 0xB:
    return MidiEventType_Controller;

  case 0xC:
    return MidiEventType_ProgramChange;

  case 0xD:
    return MidiEventType_ChannelPressure;

  case 0xE:
    return MidiEventType_PitchWheel;

  default:
    return MidiEventType_Unknown;
  }
}

MidiMetaEventType MidiEvent::MetaType() const {
  if (Type() != MidiEventType_Meta)
    return MidiMetaEvent_Unknown;

  return static_cast<MidiMetaEventType>(m_meta_type);
}

bool MidiEvent::IsEnd() const {
  return (Type() == MidiEventType_Meta &&
          MetaType() == MidiMetaEvent_EndOfTrack);
}

unsigned char MidiEvent::Channel() const {
  // The channel is held in the lower nibble of the status code
  return (m_status & 0x0F);
}

void MidiEvent::SetChannel(unsigned char channel) {
  if (channel > 15)
    return;

  // Clear out the old channel
  m_status = m_status & 0xF0;

  // Set the new channel
  m_status = m_status | channel;
}

void MidiEvent::SetVelocity(int velocity) {
  if (Type() != MidiEventType_NoteOn)
    return;

  m_data2 = static_cast<unsigned char>(velocity);
}

bool MidiEvent::HasText() const {
  if (Type() != MidiEventType_Meta)
    return false;

  switch (m_meta_type) {
  case MidiMetaEvent_Text:
  case MidiMetaEvent_Copyright:
  case MidiMetaEvent_TrackName:
  case MidiMetaEvent_Instrument:
  case MidiMetaEvent_Lyric:
  case MidiMetaEvent_Marker:
  case MidiMetaEvent_Cue:
  case MidiMetaEvent_PatchName:
  case MidiMetaEvent_DeviceName:
    return true;

  default:
    return false;
  }
}

NoteId MidiEvent::NoteNumber() const {
  if (Type() != MidiEventType_NoteOn &&
      Type() != MidiEventType_NoteOff)
    return 0;

  return m_data1;
}

void MidiEvent::ShiftNote(int shift_amount) {
  if (Type() != MidiEventType_NoteOn &&
      Type() != MidiEventType_NoteOff)
    return;

  m_data1 = m_data1 + static_cast<unsigned char>(shift_amount);
}

int MidiEvent::ProgramNumber() const {
  if (Type() != MidiEventType_ProgramChange)
    return 0;

  return m_data1;
}

string MidiEvent::NoteName(unsigned int note_number) {
  // Music domain knowledge
  const static unsigned int NotesPerOctave = 12;
  const static string NoteBases[NotesPerOctave] = {
    STRING("C"),  STRING("C#"), STRING("D"),
    STRING("D#"), STRING("E"),  STRING("F"),
    STRING("F#"), STRING("G"),  STRING("G#"),
    STRING("A"),  STRING("A#"), STRING("B")
  };

  unsigned int octave = (note_number / NotesPerOctave);
  const string note_base = NoteBases[note_number % NotesPerOctave];

  return STRING(note_base << octave);
}

int MidiEvent::NoteVelocity() const {
  if (Type() == MidiEventType_NoteOff)
    return 0;

  if (Type() != MidiEventType_NoteOn)
    return -1;

  return static_cast<int>(m_data2);
}

string MidiEvent::Text() const {
  if (!HasText())
    return "";

  return m_text;
}

unsigned long MidiEvent::GetTempoInUsPerQn() const {
  if (Type() != MidiEventType_Meta ||
      MetaType() != MidiMetaEvent_TempoChange)
    throw MidiError(MidiError_RequestedTempoFromNonTempoEvent);

  return m_tempo_uspqn;
}
