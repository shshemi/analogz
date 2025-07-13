from analogz import Buffer, LineIter, Line, ArcStr


def test_buffer_init():
    buff = Buffer("Line1\nLine2\nLine3")
    assert buff is not None

def test_buffer_to_string():
    content = "Line1\nLine2\nLine3"
    buff = Buffer(content)
    assert str(buff) == content

def test_buffer_getitem():
    buff = Buffer("Line1\nLine2\nLine3")
    assert isinstance(buff[0], Line)
    assert str(buff[0]) == "Line1"

    assert isinstance(buff[1], Line)
    assert str(buff[1]) == "Line2"

    assert isinstance(buff[2], Line)
    assert str(buff[2]) == "Line3"

def test_buffer_iter():
    buff = Buffer("Line1\nLine2\nLine3")
    itr = iter(buff)
    assert isinstance(itr, LineIter)
    assert str(next(itr)) == "Line1"
    assert str(next(itr)) == "Line2"
    assert str(next(itr)) == "Line3"

    for line in buff:
        assert isinstance(line, Line)

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

def test_line_start_stop_find():
    buff = Buffer("Line1\nLine2\nLine3")
    line0 = buff[0]
    line1 = buff[1]
    assert isinstance(line0, Line)
    assert isinstance(line1, Line)

    found = line0.find("ne1")
    assert isinstance(found, ArcStr)
    assert found.start == 2 and found.stop == 5

    found = line1.find("ne2")
    assert isinstance(found, ArcStr)
    assert found.start == 8 and found.stop == 11

    not_found = line0.find("ne2")
    assert not_found is None
