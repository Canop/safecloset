
# Sponsorship

**SafeCloset** is developed by [Denys Séguret](https://twitter.com/DenysSeguret), also known as [Canop](https://github.com/Canop) or [dystroy](https://dystroy.org).

**SafeCloset** is free for everybody, and everbody's welcome to ask for advice or new features. But if you *want* to help me find time to improve it, and to develop [other free programs](https://dystroy.org), consider [sponsoring me](https://github.com/sponsors/Canop). There's absolutely no moral obligation to do so, though.

# Chat

The best place to chat about safecloset, to talk about features or bugs, is the Miaou chat.

* [Rust & SafeCloset room on Miaou](https://miaou.dystroy.org/3490?rust)
* [French Programmers room on Miaou](https://miaou.dystroy.org/3)

# Issues

We use [GitHub's issue manager](https://github.com/Canop/safecloset/issues).

Before posting a new issue, check your problem hasn't already been raised and in case of doubt **please come first discuss it on the chat**.

# Independant security audit

None has been done yet.
I'd welcome help on this topic.

# Storage format

The storage format is described to ensure it's possible to replace SafeCloset with another software if needed.

The closet file is a [MessagePack](https://msgpack.org/index.html) encoded structure `Closet` with the following fields:

* `comments`: a string (optional, absent in closets written before 0.6.0)
* `salt`: a string
* `drawers`: an array of `ClosedDrawer`

The MessagePack serialization preserves field names and allows future additions.

Beware that every sequence of bytes in the format (`nonce`, `content`, `garbage`, and the `bytes` of a `DrawerId`) is encoded as a MessagePack **array of integers**, and not with the MessagePack `bin` format.

An instance of `ClosedDrawer` is a structure with the following fields:

* `id`: an instance of `DrawerId`
* `nonce`: a sequence of 12 bytes
* `content`: a sequence of bytes

An instance of `DrawerId` is a structure with a single field:

* `bytes`: a sequence of 20 bytes

The `content` is the AES-GCM-SIV encryption of the serialized drawer with the included `nonce`.
The key used for this encryption is the 256 bits Argon2 hash of the UTF-8 bytes of the password, with the UTF-8 bytes of the closet's `salt` as Argon2 salt, and those parameters:

* variant: Argon2i
* version: 0x13
* hash length: 32 bytes
* iterations (time cost): 3
* memory cost: 4096 KiB
* lanes: 1
* no secret, no associated data

The serialized drawer is a MessagePack encoded structure with the following fields:

* `id`: an instance of `DrawerId`, which must be equal to the `id` of the enclosing `ClosedDrawer`
* `entries`: an array of `Entry`
* `settings`: an instance of `DrawerSettings`
* `closet`: a deeper closet, containing drawers, etc.
* `garbage`: a random sequence of bytes, rewritten at every save

Instances of `Entry` contain the following fields:

* `name`: a string
* `value`: a string

Instances of `DrawerSettings` contain the following fields:

* `hide_values`: a boolean
* `open_all_values`: a boolean (optional, false if not present)
* `values_as_markdown`: a boolean (optional, false if not present)

