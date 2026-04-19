import unittest

import server


class MultipartParserTest(unittest.TestCase):
    def test_preserves_binary_file_bytes(self):
        boundary = b"dastill-boundary"
        audio = b"\r\nID3 podcast bytes\r\n"
        body = (
            b"--"
            + boundary
            + b"\r\n"
            + b'Content-Disposition: form-data; name="file"; filename="episode.mp3"\r\n'
            + b"Content-Type: audio/mpeg\r\n\r\n"
            + audio
            + b"\r\n--"
            + boundary
            + b"--\r\n"
        )

        parsed, filename = server.parse_multipart_file(body, boundary)

        self.assertEqual(parsed, audio)
        self.assertEqual(filename, "episode.mp3")

    def test_parses_audio_url_field(self):
        boundary = b"dastill-boundary"
        body = (
            b"--"
            + boundary
            + b"\r\n"
            + b'Content-Disposition: form-data; name="audio_url"\r\n\r\n'
            + b"https://example.com/audio.mp3"
            + b"\r\n--"
            + boundary
            + b"--\r\n"
        )

        fields, files = server.parse_multipart(body, boundary)

        self.assertEqual(fields["audio_url"], "https://example.com/audio.mp3")
        self.assertEqual(files, {})

    def test_rejects_private_audio_url_hosts(self):
        with self.assertRaises(server.AudioFetchError):
            server.validate_public_url("http://127.0.0.1/audio.mp3")


if __name__ == "__main__":
    unittest.main()
