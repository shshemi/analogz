import analogz


def test_sample():
    buff = analogz.Buffer("Line1\nLine2\nLine3")
    assert str(buff[0]) == "Line1"
    assert str(buff[1]) == "Line2"
    assert str(buff[2]) == "Line3"

    # Iterator test
    lines = [str(line) for line in buff]
    assert lines == ["Line1", "Line2", "Line3"]

    # Slice test
    sl = buff[1:4]
    assert [str(line) for line in sl] == ["Line2", "Line3"]
