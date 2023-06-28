#include <cdda_interface.h>
#include <cdda_paranoia.h>

#include <stdbool.h>

/* Wrapper for IS_AUDIO macro */
bool is_audio(const cdrom_drive *d, int i)
{
    return IS_AUDIO(d, i);
}
