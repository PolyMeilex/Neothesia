noinst_LTLIBRARIES = libmidi.la
dist_libmidi_la_SOURCES =	libmidi/Midi.cpp \
				libmidi/MidiUtil.cpp \
				libmidi/MidiTrack.cpp \
				libmidi/MidiEvent.cpp
noinst_HEADERS =		libmidi/MidiEvent.h \
				libmidi/Midi.h \
				libmidi/MidiTrack.h \
				libmidi/MidiTypes.h \
				libmidi/MidiUtil.h \
				libmidi/Note.h

libmidi_la_CPPFLAGS = -iquote $(srcdir)/libmidi

bin_PROGRAMS = neothesia
dist_neothesia_SOURCES =	CompatibleSystem.cpp \
				KeyboardDisplay.cpp \
				MenuLayout.cpp \
				Renderer.cpp \
				TextWriter.cpp \
				TrackSelectionState.cpp \
				DeviceTile.cpp \
				FileSelector.cpp \
				LinthesiaError.cpp \
				MidiComm.cpp \
				StatsState.cpp \
				Tga.cpp \
				TrackTile.cpp \
				GameState.cpp \
				main.cpp \
				PlayingState.cpp \
				StringTile.cpp \
				TitleState.cpp \
				UserSettings.cpp \
				GLShader.cpp
noinst_HEADERS +=		CompatibleSystem.h \
				DeviceTile.h \
				FileSelector.h \
				FrameCounter.h \
				GameState.h \
				KeyboardDisplay.h \
				LinthesiaError.h \
				MenuLayout.h \
				MidiComm.h \
				OSGraphics.h \
				PlayingState.h \
				Renderer.h \
				SharedState.h \
				StatsState.h \
				StringTile.h \
				StringUtil.h \
				Textures.h \
				TextWriter.h \
				Tga.h \
				TitleState.h \
				TrackProperties.h \
				TrackSelectionState.h \
				TrackTile.h \
				UserSettings.h \
				Version.h \
				GLShader.h

neothesia_LDFLAGS = @GTKMM_LIBS@ @GCONFMM_LIBS@ @GTKGLEXTMM_LIBS@ @ALSA_LIBS@
neothesia_LDADD = libmidi.la
neothesia_CXXFLAGS = -std=c++11 -iquote $(srcdir)/libmidi -lGLEW
neothesia_CPPFLAGS = @GTKMM_CFLAGS@ @GCONFMM_CFLAGS@ @GTKGLEXTMM_CFLAGS@ @ALSA_CFLAGS@ -DGRAPHDIR="\"${graphdir}\"" -DSCRIPTDIR="\"${scriptdir}\""

ctags-dependencies:
	@$(CC) -M -std=c++11 -I/usr/include @GTKMM_CFLAGS@ @GCONFMM_CFLAGS@ @GTKGLEXTMM_CFLAGS@ @ALSA_CFLAGS@ \
		$(dist_libmidi_la_SOURCES) $(noinst_HEADERS) $(dist_neothesia_SOURCES) 2>/dev/null | \
		sed 's/^.*://;s/^ //;s/ \\//;s/ /\n/g' | \
		sort -u | \
		$(CTAGS) --tag-relative=yes -L - --c++-kinds=+p --fields=+iaS --extra=+q -f $(top_srcdir)/tags

.PHONY:ctags-dependencies
