"""Codec for the daemon's GraphQL ``ID`` scalar (``NodeIds`` in cdx-daemon).

A node id is a chain of ``(typeName, id)`` segments, root first. Each segment is
``varint(len(typeName)) typeName varint(len(id)) id``; the whole chain is
URL-safe base64 without padding. A ``Project`` id is a single root segment
carrying the project UUID, so the CLI can turn the ids the daemon returns back
into UUIDs and build them for its own queries.
"""

import base64
import binascii


def _write_varint(buf, value):
    while value >= 0x80:
        buf.append((value & 0x7F) | 0x80)
        value >>= 7
    buf.append(value)


def _read_varint(data, idx):
    result = 0
    shift = 0
    while True:
        if idx >= len(data) or shift >= 64:
            raise ValueError("truncated varint")
        byte = data[idx]
        idx += 1
        result |= (byte & 0x7F) << shift
        if byte & 0x80 == 0:
            return result, idx
        shift += 7


def _read_chunk(data, idx):
    length, idx = _read_varint(data, idx)
    if idx + length > len(data):
        raise ValueError("truncated segment")
    return data[idx : idx + length].decode("utf-8"), idx + length


def encode(*segments):
    """Encode ``(typeName, id)`` segments, root first, into a node id string."""
    if not segments:
        raise ValueError("at least one segment is required")
    buf = bytearray()
    for type_name, ident in segments:
        for part in (type_name, ident or ""):
            raw = part.encode("utf-8")
            _write_varint(buf, len(raw))
            buf.extend(raw)
    return base64.urlsafe_b64encode(bytes(buf)).decode("ascii").rstrip("=")


def decode(encoded):
    """Decode a node id into its ``[(typeName, id), ...]`` segments, root first.

    Raises ``ValueError`` when the string is not a node id.
    """
    if not encoded or not isinstance(encoded, str):
        raise ValueError("node id must be a non-empty string")
    padded = encoded + "=" * (-len(encoded) % 4)
    try:
        data = base64.urlsafe_b64decode(padded)
    except (binascii.Error, ValueError) as exc:
        raise ValueError(f"not a node id: {encoded}") from exc
    segments = []
    idx = 0
    while idx < len(data):
        type_name, idx = _read_chunk(data, idx)
        ident, idx = _read_chunk(data, idx)
        segments.append((type_name, ident))
    if not segments:
        raise ValueError(f"not a node id: {encoded}")
    return segments


def root_id(encoded, type_name):
    """The ``id`` of a single-segment node id of ``type_name``, else ``None``."""
    try:
        segments = decode(encoded)
    except ValueError:
        return None
    if len(segments) != 1 or segments[0][0] != type_name:
        return None
    return segments[0][1]
