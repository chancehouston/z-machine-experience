/* C bindings to the experience layer of the Z-Machine Experience project.
 *
 * Contract: specs/004-first-demo/contracts/c-abi.md.
 *
 * Ownership: a session is created by zmx_session_new and destroyed exactly once
 * by zmx_session_free. A string returned by this interface is destroyed exactly
 * once by zmx_string_free. A session may be used from any thread, but not from
 * two at once.
 */
#ifndef ZMX_H
#define ZMX_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct zmx_session zmx_session;

/* Loads a story file and runs it to its first stop. Copies the bytes.
 * With has_seed non-zero, seed (0 to 2147483647) fixes the game's randomness;
 * otherwise entropy, a number from the caller's random source, is used.
 * On failure returns NULL and, if error is not NULL, sets *error to a message
 * the caller frees with zmx_string_free. */
zmx_session *zmx_session_new(const uint8_t *story, size_t length,
                             uint32_t seed, int has_seed, uint32_t entropy,
                             char **error);

/* As zmx_session_new, with the text of a game pack for the story
 * (docs/introspection.md, section 12): a NUL-terminated UTF-8 string, or NULL
 * for none. The text is read before the call returns. A pack that cannot be
 * read, or that is another story's, changes nothing and is not an error. */
zmx_session *zmx_session_new_with_pack(const uint8_t *story, size_t length,
                                       uint32_t seed, int has_seed, uint32_t entropy,
                                       const char *pack, char **error);

/* Destroys a session. NULL is allowed. */
void zmx_session_free(zmx_session *session);

/* Each names the game and turn the caller was showing (the snapshot's
 * generation and turn_count). Each returns 1 if the game received a line and
 * 0 if the command was refused. */
int zmx_session_send_line(zmx_session *session, uint64_t generation, size_t turn_count,
                          const char *utf8_line);
int zmx_session_tap(zmx_session *session, uint64_t generation, size_t turn_count,
                    const char *place);
int zmx_session_tap_extra(zmx_session *session, uint64_t generation, size_t turn_count,
                          size_t index);

/* Abandons the game and starts a new one, with a fresh number from the caller. */
void zmx_session_new_game(zmx_session *session, uint32_t entropy);

/* The snapshot as JSON, with the turns from `since` onward. The caller frees it
 * with zmx_string_free. NULL if session is NULL. */
char *zmx_session_snapshot(zmx_session *session, size_t since);

/* Destroys a string this interface returned. NULL is allowed. */
void zmx_string_free(char *string);

#ifdef __cplusplus
}
#endif

#endif /* ZMX_H */
