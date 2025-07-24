from analogz import Buffer, LineIter, ArcStr, Regex


def test_buffer_init():
    buff = Buffer("Line1\nLine2\nLine3")
    assert buff is not None

def test_buffer_to_string():
    content = "Line1\nLine2\nLine3"
    buff = Buffer(content)
    assert str(buff) == content

def test_buffer_getitem():
    buff = Buffer("Line1\nLine2\nLine3")
    assert isinstance(buff[0], ArcStr)
    assert str(buff[0]) == "Line1"

    assert isinstance(buff[1], ArcStr)
    assert str(buff[1]) == "Line2"

    assert isinstance(buff[2], ArcStr)
    assert str(buff[2]) == "Line3"

def test_buffer_iter():
    buff = Buffer("Line1\nLine2\nLine3")
    itr = iter(buff)
    assert isinstance(itr, LineIter)
    assert str(next(itr)) == "Line1"
    assert str(next(itr)) == "Line2"
    assert str(next(itr)) == "Line3"

    for line in buff:
        assert isinstance(line, ArcStr)

    lines = [str(line) for line in buff]
    assert lines == ["Line1", "Line2", "Line3"]

def test_buffer_getitem_slice1():
    buff = Buffer("Line1\nLine2\nLine3")
    sb = buff[1:4]
    assert isinstance(sb, Buffer)
    assert [str(line) for line in sb] == ["Line2", "Line3"]

def test_buffer_getitem_slice2():
    buff = Buffer("Line1\nLine2\nLine3")
    sb = buff[1:]
    assert isinstance(sb, Buffer)
    assert [str(line) for line in sb] == ["Line2", "Line3"]

def test_buffer_getitem_slice3():
    buff = Buffer("Line1\nLine2\nLine3")
    sb = buff[:2]
    assert isinstance(sb, Buffer)
    assert [str(line) for line in sb] == ["Line1","Line2"]

def test_line_find_str():
    buff = Buffer("Line1\nLine2\nLine3")
    line0 = buff[0]
    line1 = buff[1]
    assert isinstance(line0, ArcStr)
    assert isinstance(line1, ArcStr)

    found = line0.find("ne1")
    assert isinstance(found, ArcStr)
    assert found.start == 2 and found.stop == 5

    found = line1.find("ne2")
    assert isinstance(found, ArcStr)
    assert found.start == 8 and found.stop == 11

    not_found = line0.find("ne2")
    assert not_found is None

def test_line_find_regex():
    buff = Buffer("Line1\nLine2\nLine3")
    line0 = buff[0]
    line1 = buff[1]
    assert isinstance(line0, ArcStr)
    assert isinstance(line1, ArcStr)

    regex0 = Regex(r"\d+")
    found = regex0.find(line0)
    assert isinstance(found, ArcStr)
    assert found.start == 4 and found.stop == 5

    regex1 = Regex(r"[A-Z]")
    found = regex1.find(line1)
    assert isinstance(found, ArcStr)
    assert found.start == 6 and found.stop == 7

    regex2 = Regex("[A-Z]{2}")
    not_found = regex2.find(line0)
    assert not_found is None
