// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __MIDI_UTILS_H
#define __MIDI_UTILS_H

#include <iostream>
#include <string>

// Handy string macros
#ifndef STRING
#include <sstream>
#define STRING(v) ((static_cast<std::ostringstream&>(std::ostringstream().flush() << v)).str())
#endif

// Cross-platform Endian conversion functions
//
// MIDI is big endian.  Some platforms aren't
unsigned long BigToSystem32(unsigned long x);
unsigned short BigToSystem16(unsigned short x);

// MIDI contains these wacky variable length numbers where
// the value is stored only in the first 7 bits of each
// byte, and the last bit is a kind of "keep going" flag.
unsigned long parse_variable_length(std::istream &in);

const static int InstrumentCount = 130;
const static int InstrumentIdVarious = InstrumentCount - 1;
const static int InstrumentIdPercussion = InstrumentCount - 2;
extern std::string const InstrumentNames[InstrumentCount];

enum MidiErrorCode {

   MidiError_BadFilename,
   MidiError_NoHeader,
   MidiError_UnknownHeaderType,
   MidiError_BadHeaderSize,
   MidiError_Type2MidiNotSupported,
   MidiError_BadType0Midi,
   MidiError_SMTPETimingNotImplemented,

   MidiError_TrackHeaderTooShort,
   MidiError_BadTrackHeaderType,
   MidiError_TrackTooShort,
   MidiError_BadTrackEnd,

   MidiError_EventTooShort,
   MidiError_UnknownEventType,
   MidiError_UnknownMetaEventType,

   // MMSYSTEM Errors for MIDI I/O
   MidiError_MM_NoDevice,
   MidiError_MM_NotEnabled,
   MidiError_MM_AlreadyAllocated,
   MidiError_MM_BadDeviceID,
   MidiError_MM_InvalidParameter,
   MidiError_MM_NoDriver,
   MidiError_MM_NoMemory,
   MidiError_MM_Unknown,

   MidiError_NoInputAvailable,
   MidiError_MetaEventOnInput,

   MidiError_InputError,
   MidiError_InvalidInputErrorBehavior,

   MidiError_RequestedTempoFromNonTempoEvent
};

class MidiError : public std::exception {
public:
   MidiError(MidiErrorCode error) : m_error(error) { }
   std::string GetErrorDescription() const;

   const MidiErrorCode m_error;

private:
   MidiError operator =(const MidiError&);
};

enum MidiEventType {

   MidiEventType_Meta,
   MidiEventType_SysEx,
   MidiEventType_Unknown,

   MidiEventType_NoteOff,
   MidiEventType_NoteOn,
   MidiEventType_Aftertouch,
   MidiEventType_Controller,
   MidiEventType_ProgramChange,
   MidiEventType_ChannelPressure,
   MidiEventType_PitchWheel
};

std::string GetMidiEventTypeDescription(MidiEventType type);

enum MidiMetaEventType {

   MidiMetaEvent_SequenceNumber = 0x00,

   MidiMetaEvent_Text = 0x01,
   MidiMetaEvent_Copyright = 0x02,
   MidiMetaEvent_TrackName = 0x03,
   MidiMetaEvent_Instrument = 0x04,
   MidiMetaEvent_Lyric = 0x05,
   MidiMetaEvent_Marker = 0x06,
   MidiMetaEvent_Cue = 0x07,
   MidiMetaEvent_PatchName = 0x08,
   MidiMetaEvent_DeviceName = 0x09,

   MidiMetaEvent_EndOfTrack = 0x2F,
   MidiMetaEvent_TempoChange = 0x51,
   MidiMetaEvent_SMPTEOffset = 0x54,
   MidiMetaEvent_TimeSignature = 0x58,
   MidiMetaEvent_KeySignature = 0x59,

   MidiMetaEvent_Proprietary = 0x7F,

   // Deprecated Meta Events
   MidiMetaEvent_ChannelPrefix = 0x20,
   MidiMetaEvent_MidiPort = 0x21,

   MidiMetaEvent_Unknown = 0xFF
};

// Returns a human-readable description of this meta type
// type type of the text ought to contain in
// this event. (e.g. Copyright, Lyric, Track name, etc.)
// (If this isn't a meta event, returns an empty string)
std::string GetMidiMetaEventTypeDescription(MidiMetaEventType type);

#endif
