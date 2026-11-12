# Contract: The C Interface

**Crate**: `zmachine-ffi` | **Header**: `crates/zmachine-ffi/include/zmx.h` | **Plan**:
[../plan.md](../plan.md)

```c
typedef struct zmx_session zmx_session;

/* Loads a story file and runs it to its first stop. Copies the bytes.
   has_seed 0 draws a fresh seed. On failure returns NULL and, if error is not
   NULL, sets *error to a message the caller frees with zmx_string_free. */
zmx_session *zmx_session_new(const uint8_t *story, size_t length,
                             uint32_t seed, int has_seed, char **error);

void zmx_session_free(zmx_session *session);            /* NULL is allowed */

/* Each returns 1 if the game received a line, 0 if it was refused. */
int zmx_session_send_line(zmx_session *session, const char *utf8_line);
int zmx_session_tap(zmx_session *session, const char *place);   /* "north" … "out" */
int zmx_session_tap_extra(zmx_session *session, size_t index);

void zmx_session_restart(zmx_session *session);

/* The snapshot as JSON, with the turns from `since` onward. The caller frees it. */
char *zmx_session_snapshot(const zmx_session *session, size_t since);

void zmx_string_free(char *string);                     /* NULL is allowed */
```

## Rules

1. **Ownership.** A session is created by `zmx_session_new` and destroyed exactly once by
   `zmx_session_free`. A string returned by this interface is destroyed exactly once by
   `zmx_string_free`. Nothing else is ever freed by the caller.
2. **Null is never dereferenced.** A null session makes every function return 0, null or
   nothing. A null or non-UTF-8 line or place is refused with 0.
3. **One thread at a time.** A session may be used from any thread but not from two at
   once. The caller arranges that; the app does so with an actor.
4. **No panic crosses the boundary.** Each function catches a panic and reports failure;
   the session is then in the `fault` state.
5. **Every `unsafe` block has a comment** stating what the caller promised that makes it
   sound.
6. **The header and the library agree**: a test checks that every function the header
   declares is exported, and no other symbol beginning `zmx_` is.
