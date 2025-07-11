from typing import Optional, Union
from ._lib_rs import PyBuffer, PyLine, PyLineIter

class Line:
    __slots__ = ["__line"]

    def __init__(self, line: PyLine):
        self.__line = line

    def __str__(self) -> str:
        return self.__line.to_string()

class LineIter:
    __slots__ = ["__iter"]

    def __init__(self, line_iter: PyLineIter):
        self.__iter = line_iter

    def __iter__(self):
        return self

    def __next__(self):
        return self.__iter.next()

class Buffer:
    __slots__ = ["__buffer"]

    def __init__(self, content: str):
        self.__buffer = PyBuffer(content)


    def __iter__(self) -> LineIter:
        return LineIter(self.__buffer.iter())

    def __getitem__(self, idx) -> Union["Buffer", Line]:
        if isinstance(idx, slice):
            assert slice.step is None, "Step is not supported"
            buf = Buffer.__new__(Buffer)
            buf.__buffer = self.__buffer.slice(idx.start, idx.stop)
            return buf
        elif isinstance(idx, int):
            return Line(self.__buffer.get(idx))
        else:
            raise IndexError(f"Invalid index type: {type(idx)}")
