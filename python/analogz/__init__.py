import functools
from typing import Optional, Union
from ._lib_rs import PyBuffer, PyLineIter, PyArcStr, PyCompiledRegex

class ArcStr:
    __slots__ = ["__arc_str"]

    def __init__(self, arc_str: PyArcStr):
        self.__arc_str = arc_str

    @property
    def start(self) -> int:
        return self.__arc_str.start()

    @property
    def stop(self) -> int:
        return self.__arc_str.end()

    def find_str(self, pattern: str) -> Optional["ArcStr"]:
        astr = self.__arc_str.find_str(pattern)
        if astr is None:
            return None
        return ArcStr(astr)

    def find_regex(self, pattern: str) -> Optional["ArcStr"]:
        pattern = compile_regex(pattern)
        astr = self.__arc_str.find_regex(pattern)
        if astr is None:
            return None
        return ArcStr(astr)

    def __str__(self) -> str:
        return self.__arc_str.to_string()


class LineIter:
    __slots__ = ["__iter"]

    def __init__(self, line_iter: PyLineIter):
        self.__iter = line_iter

    def __iter__(self):
        return self

    def __next__(self):
        next = self.__iter.next()
        if next is None:
            raise StopIteration()
        return ArcStr(next)

class Buffer:
    __slots__ = ["__buffer"]

    def __init__(self, content: str):
        self.__buffer = PyBuffer(content)

    def __iter__(self) -> LineIter:
        return LineIter(self.__buffer.iter())

    def __str__(self) -> str:
        return self.__buffer.to_string()

    def __getitem__(self, idx) -> Union["Buffer", ArcStr]:
        if isinstance(idx, slice):
            assert slice.step is not None, "Step is not supported"
            buf = Buffer.__new__(Buffer)
            buf.__buffer = self.__buffer.slice(idx.start, idx.stop)
            return buf
        elif isinstance(idx, int):
            return ArcStr(self.__buffer.get(idx))
        else:
            raise IndexError(f"Invalid index type: {type(idx)}")


functools.lru_cache(maxsize=1024)
def compile_regex(pattern: str) -> PyCompiledRegex:
    return PyCompiledRegex(pattern)
