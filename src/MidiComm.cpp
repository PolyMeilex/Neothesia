// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <string>
#include <sstream>
#include <alsa/asoundlib.h>

#include "libmidi/MidiEvent.h"
#include "libmidi/MidiUtil.h"

#include "MidiComm.h"
#include "UserSettings.h"
#include "CompatibleSystem.h"
#include "StringUtil.h"

using namespace std;

// ALSA sequencer descriptor
static bool midi_initiated = false;
static bool emulate_kb = false;
static snd_seq_t* alsa_seq;

// ALSA ports
static int local_out, local_in, anon_in, keybd_out = -1;

void midiInit() {

  if (midi_initiated)
    return;

  int err = snd_seq_open(&alsa_seq, "default", SND_SEQ_OPEN_DUPLEX, 0);
  int ownid = snd_seq_client_id(alsa_seq);
  midi_initiated = true;

  // Could not open sequencer, no out devices
  if (err < 0) {
    alsa_seq = NULL;
    Compatible::ShowError("Could not open MIDI sequencer. No MIDI available");
    return;
  }

  snd_seq_set_client_name(alsa_seq, "Linthesia");

  // meanings of READ and WRITE are permissions of the port from the viewpoint of other ports
  // READ: port allows to send events to other ports
  local_out = snd_seq_create_simple_port(alsa_seq, "Linthesia Output",
                                         SND_SEQ_PORT_CAP_READ | SND_SEQ_PORT_CAP_SUBS_READ,
                                         SND_SEQ_PORT_TYPE_MIDI_GENERIC);

  keybd_out = snd_seq_create_simple_port(alsa_seq, "Linthesia Keyboard",
                                         SND_SEQ_PORT_CAP_READ | SND_SEQ_PORT_CAP_SUBS_READ,
                                         SND_SEQ_PORT_TYPE_MIDI_GENERIC);

  // WRITE: port allows to receive events from other ports
  local_in = snd_seq_create_simple_port(alsa_seq, "Linthesia Input",
                                        SND_SEQ_PORT_CAP_WRITE | SND_SEQ_PORT_CAP_SUBS_WRITE,
                                        SND_SEQ_PORT_TYPE_MIDI_GENERIC);

  anon_in = snd_seq_create_simple_port(alsa_seq, "Linthesia Annonce Listener",
                                       SND_SEQ_PORT_CAP_WRITE | SND_SEQ_PORT_CAP_NO_EXPORT,
                                       SND_SEQ_PORT_TYPE_MIDI_GENERIC | SND_SEQ_PORT_TYPE_APPLICATION);

   if (anon_in < 0)
       return; // handle error

  // Subscribe on port opening
  snd_seq_port_subscribe_t *sub;
  snd_seq_addr_t sender, dest;

  snd_seq_port_subscribe_alloca(&sub);
  // Receive events from annoncer's port
  sender.client = SND_SEQ_CLIENT_SYSTEM;
  sender.port = SND_SEQ_PORT_SYSTEM_ANNOUNCE;
  snd_seq_port_subscribe_set_sender(sub, &sender);
  // Forward them to our port
  dest.client = ownid;
  dest.port = anon_in;
  snd_seq_port_subscribe_set_dest(sub, &dest);
  err = snd_seq_subscribe_port(alsa_seq, sub);
  if (err<0) {
      fprintf(stderr, "Cannot subscribe announce port: %s\n", snd_strerror(err));
      return;
  }

}

void midiStop() {

  snd_seq_close(alsa_seq);
}

void sendNote(const unsigned char note, bool on) {

  if (emulate_kb) {
    snd_seq_event_t ev;
    snd_seq_ev_clear(&ev);

    snd_seq_ev_set_source(&ev, keybd_out);
    snd_seq_ev_set_subs(&ev);
    snd_seq_ev_set_direct(&ev);

    if (on)
      // velocity ~ 60 for audio preview
      snd_seq_ev_set_noteon(&ev, 0, note, 60);
    else
      snd_seq_ev_set_noteoff(&ev, 0, note, 0);

    snd_seq_event_output(alsa_seq, &ev);
    snd_seq_drain_output(alsa_seq);
  }
}

// private use
void doRetrieveDevices(unsigned int perms, MidiCommDescriptionList& devices) {

  midiInit();
  if (alsa_seq == NULL)
    return;

  snd_seq_client_info_t* cinfo;
  snd_seq_port_info_t* pinfo;
  int count = 0, ownid = snd_seq_client_id(alsa_seq);

  snd_seq_client_info_alloca(&cinfo);
  snd_seq_port_info_alloca(&pinfo);
  snd_seq_client_info_set_client(cinfo, -1);

  while (snd_seq_query_next_client(alsa_seq, cinfo) >= 0) {

    // reset query info
    snd_seq_port_info_set_client(pinfo, snd_seq_client_info_get_client(cinfo));
    snd_seq_port_info_set_port(pinfo, -1);

    while (snd_seq_query_next_port(alsa_seq, pinfo) >= 0) {
      if ((snd_seq_port_info_get_capability(pinfo) & perms) == perms) {

        int client = snd_seq_client_info_get_client(cinfo);
        int port = snd_seq_port_info_get_port(pinfo);

        // filter own ports
        if (client == ownid && (port == local_in || port == local_out))
          continue;

        MidiCommDescription d;
        d.id = count++;
        d.name = snd_seq_port_info_get_name(pinfo);
        d.client = client;
        d.port = port;

        devices.push_back(d);
      }
    }
  }
}

// Midi IN Ports

static bool built_input_list = false;
static MidiCommDescriptionList in_list(MidiCommIn::GetDeviceList());

MidiCommIn::MidiCommIn(unsigned int device_id) {
  m_should_reconnect = false;

  m_description = GetDeviceList()[device_id];

  // Connect local in to selected port
  int res = snd_seq_connect_from(alsa_seq, local_in, m_description.client, m_description.port);
  if (res < 0) {
    string msg = snd_strerror(res);
    cout << "[WARNING] Input, cannot connect from '" << m_description.name << "': " << msg << endl;
  }

  // enable internal keyboard
  if (m_description.client == snd_seq_client_id(alsa_seq) and
      m_description.port == keybd_out)
    emulate_kb = true;
}

MidiCommIn::~MidiCommIn() {

  // Disconnect local in to selected port
  snd_seq_disconnect_from(alsa_seq, local_in, m_description.client, m_description.port);
}

MidiCommDescriptionList MidiCommIn::GetDeviceList() {

  if (built_input_list)
    return in_list;

  built_input_list = true;
  MidiCommDescriptionList devices;

  unsigned int perms = SND_SEQ_PORT_CAP_READ|SND_SEQ_PORT_CAP_SUBS_READ;
  doRetrieveDevices(perms, devices);

  return devices;
}

void MidiCommIn::UpdateDeviceList()
{
    built_input_list = false;
    in_list = MidiCommIn::GetDeviceList();
}

MidiEvent MidiCommIn::Read() {

  if (snd_seq_event_input_pending(alsa_seq, 1) < 1)
    return MidiEvent::NullEvent();

  MidiEventSimple simple;
  snd_seq_event_t* ev;
  snd_seq_event_input(alsa_seq, &ev);

  switch(ev->type) {
  case SND_SEQ_EVENT_NOTEON:
    simple.status = 0x90 | (ev->data.note.channel & 0x0F); // Type and Channel
    simple.byte1 = ev->data.note.note;                     // Note number
    simple.byte2 = ev->data.note.velocity;                 // Velocity
    break;

  case SND_SEQ_EVENT_NOTEOFF:
    simple.status = 0x80 | (ev->data.note.channel & 0x0F); // Type and Channel
    simple.byte1 = ev->data.note.note;                     // Note number
    simple.byte2 = 0;                                      // Velocity
    break;

  case SND_SEQ_EVENT_PGMCHANGE:
    simple.status = 0xC0 | (ev->data.note.channel & 0x0F); // Type and Channel
    simple.byte1 = ev->data.control.value;                 // Program number
    break;

  case SND_SEQ_EVENT_PORT_EXIT:
    // USB device is disconnected - the input client is closed
    {
    cout << "MIDI device is lost" << endl;
    int lost_client = ev->data.addr.client;
    int lost_port   = ev->data.addr.port;
    // TODO add better error reporting
    }
    break;

  case SND_SEQ_EVENT_PORT_START:
    {
    int new_client = ev->data.addr.client;
    int new_port = ev->data.addr.port;
    snd_seq_port_info_t* pinfo;
    snd_seq_port_info_alloca(&pinfo);

    cout << "New MIDI device client=" << new_client << ", port=" << new_port << endl;
    int err = snd_seq_get_any_port_info(alsa_seq, new_client, new_port, pinfo);

    if (err < 0)
        return MidiEvent::NullEvent(); // error

    int port = snd_seq_port_info_get_port(pinfo);
    int client = snd_seq_port_info_get_client(pinfo);
    cout << "Port info client=" << client << ", port=" << port << endl;

    std::string new_name = snd_seq_port_info_get_name(pinfo);
    cout << "New MIDI device " << new_name << endl;

    m_should_reconnect = true;
    }
    break;

  // unknown type, do nothing
  default:
    return MidiEvent::NullEvent();
  }

  return MidiEvent::Build(simple);
}

bool MidiCommIn::KeepReading() const {

  return snd_seq_event_input_pending(alsa_seq, 1);
}

void MidiCommIn::Reset() {

  snd_seq_drop_input(alsa_seq);
}

bool MidiCommIn::ShouldReconnect() const {

  return m_should_reconnect;
}

void MidiCommIn::Reconnect() {
  // We assume, that the client and the port is the same after device's reconnect
  // Connect local in to selected port
  int res = snd_seq_connect_from(alsa_seq, local_in, m_description.client, m_description.port);
  m_should_reconnect = false;
}


// Midi OUT Ports

static bool built_output_list = false;
static MidiCommDescriptionList out_list(MidiCommOut::GetDeviceList());

MidiCommOut::MidiCommOut(unsigned int device_id) {

  m_description = GetDeviceList()[device_id];

  // Connect local out to selected port
  int res = snd_seq_connect_to(alsa_seq, local_out, m_description.client, m_description.port);
  if (res < 0) {
    string msg = snd_strerror(res);
    cout << "[WARNING] Output, cannot connect to '" << m_description.name
         << "': " << msg << endl;
  }
}

MidiCommOut::~MidiCommOut() {

  // Disconnect local out to selected port
  snd_seq_disconnect_to(alsa_seq, local_out, m_description.client, m_description.port);

  // This does not harm if done everytime...
  emulate_kb = false;
}

void MidiCommOut::UpdateDeviceList()
{
    built_output_list = false;
    out_list = MidiCommOut::GetDeviceList();
}

MidiCommDescriptionList MidiCommOut::GetDeviceList() {

  if (built_output_list)
    return out_list;

  built_output_list = true;
  MidiCommDescriptionList devices;

  unsigned int perms = SND_SEQ_PORT_CAP_WRITE|SND_SEQ_PORT_CAP_SUBS_WRITE;
  doRetrieveDevices(perms, devices);

  return devices;
}

void MidiCommOut::Write(const MidiEvent &out) {

  snd_seq_event_t ev;
  snd_seq_ev_clear(&ev);

  // Set my source, to all subscribers, direct delivery
  snd_seq_ev_set_source(&ev, local_out);
  snd_seq_ev_set_subs(&ev);
  snd_seq_ev_set_direct(&ev);

  // set event type
  switch (out.Type()) {
  case MidiEventType_NoteOn: {
    int ch = out.Channel();
    int note = out.NoteNumber();
    snd_seq_ev_set_noteon(&ev, ch, note, out.NoteVelocity());

    // save for reset
    notes_on.push_back(pair<int,int>(ch, note));
    break;
  }

  case MidiEventType_NoteOff: {
    int note = out.NoteNumber();
    int ch = out.Channel();
    snd_seq_ev_set_noteoff(&ev, ch, note, out.NoteVelocity());

    // remove from reset
    pair<int,int> p(ch, note);
    vector<pair<int,int> >::iterator i;
    for (i = notes_on.begin(); i != notes_on.end(); ++i) {
      if (*i == p) {
        notes_on.erase(i);
        break;
      }
    }

    break;
  }

  case MidiEventType_ProgramChange:
    snd_seq_ev_set_pgmchange(&ev, out.Channel(), out.ProgramNumber());
    break;

  // Unknown type, do nothing
  default:
    return;
  }

  snd_seq_event_output(alsa_seq, &ev);
  snd_seq_drain_output(alsa_seq);
}

void MidiCommOut::Reset() {

  // Sent Note-Off to every open note
  snd_seq_event_t ev;
  snd_seq_ev_clear(&ev);
  snd_seq_ev_set_source(&ev, local_out);
  snd_seq_ev_set_subs(&ev);
  snd_seq_ev_set_direct(&ev);

  vector<pair<int,int> >::const_iterator i;
  for (i = notes_on.begin(); i != notes_on.end(); ++i) {
    snd_seq_ev_set_noteoff(&ev, i->first, i->second, 0);

    snd_seq_event_output(alsa_seq, &ev);
    snd_seq_drain_output(alsa_seq);
  }

}

void MidiCommOut::Reconnect() {
  // We assume, that the client and the port is the same after device's reconnect
  int res = snd_seq_connect_to(alsa_seq, local_out, m_description.client, m_description.port);
}
