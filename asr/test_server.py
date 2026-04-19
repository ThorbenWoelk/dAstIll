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


if __name__ == "__main__":
    unittest.main()
