# Contract: The C Interface

**Crate**: `zmachine-ffi` | **Header**: `crates/zmachine-ffi/include/zmx.h` | **Plan**:
[../plan.md](../plan.md)

Eight functions.

```c
typedef struct zmx_session zmx_session;

/* Loads a story file and runs it to its first stop. Copies the bytes.
   With has_seed non-zero, seed (0 to 2147483647) fixes the game's randomness;
   otherwise entropy, a number from the caller's random source, is used.
   On failure returns NULL and, if error is not NULL, sets *error to a message
   the caller frees with zmx_string_free. */
zmx_session *zmx_session_new(const uint8_t *story, size_t length,
                             uint32_t seed, int has_seed, uint32_t entropy,
                             char **error);

void zmx_session_free(zmx_session *session);            /* NULL is allowed */

/* Each names the game and turn the caller was showing (from the snapshot's
   generation and turn_count). Each returns 1 if the game received a line and
   0 if the command was refused. */
int zmx_session_send_line(zmx_session *session, uint64_t generation, size_t turn_count,
                          const char *utf8_line);
int zmx_session_tap(zmx_session *session, uint64_t generation, size_t turn_count,
                    const char *place);                 /* "north" … "out" */
int zmx_session_tap_extra(zmx_session *session, uint64_t generation, size_t turn_count,
                          size_t index);

/* Abandons the game and starts a new one, with a fresh number from the caller. */
void zmx_session_new_game(zmx_session *session, uint32_t entropy);

/* The snapshot as JSON, with the turns from `since` onward. The caller frees it. */
char *zmx_session_snapshot(zmx_session *session, size_t since);

void zmx_string_free(char *string);                     /* NULL is allowed */
```

## Rules

1. **Ownership.** A session is created by `zmx_session_new` and destroyed exactly once by
   `zmx_session_free`. A string returned by this interface is destroyed exactly once by
   `zmx_string_free`. Nothing else is ever freed by the caller. Using a session after
   freeing it is the caller's error and cannot be detected.
2. **Null is never dereferenced.** A null session makes every function return 0, null or
   nothing. A null or non-UTF-8 line or place is refused with 0. A seed above
   2147483647 with `has_seed` set makes `zmx_session_new` fail with a message.
3. **One thread at a time.** A session may be used from any thread but not from two at
   once, snapshots included. The caller arranges that; the app does so with an actor.
4. **No panic crosses the boundary.** Each function catches a panic and reports failure.
   The wrapper around the session is then marked failed: every later command is refused,
   and its snapshot reports the `fault` state with a message, until `zmx_session_new_game`
   replaces the game.
5. **Every `unsafe` block has a comment** stating what the caller promised that makes it
   sound.
6. **The header and the library agree**: a test lists the library's exported symbols
   with `nm` and checks that the functions the header declares are exactly the symbols
   beginning `zmx_`.
7. **Built two ways**: as a static library for the app, and as an ordinary Rust library
   so that its tests can call the same functions.
