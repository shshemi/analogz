import functools
from typing import Optional, Union, Callable, TypeVar, List
from ._lib_rs import PyBuffer, PyLineIter, PyArcStr, PyRegex
from typing import Tuple

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

    def find(self, pattern: str) -> Optional["ArcStr"]:
        astr = self.__arc_str.find(pattern)
        if astr is None:
            return None
        return ArcStr(astr)

    def split(self, pos: int) -> Tuple["ArcStr", "ArcStr"]:
        s1, s2 =  self.__arc_str.split_at(pos)
        return ArcStr(s1), ArcStr(s2)

    def contains(self, other) -> bool:
        if not isinstance(other, PyArcStr):
            return self.__arc_str.contains(PyArcStr(other))
        return self.__arc_str.contains(other.__arc_str)

    def boundries(self) -> Tuple[int, int]:
        return self.__arc_str.boundries()

    def rel_position(self, anchor: "ArcStr") -> Optional[int]:
        return self.__arc_str.rel_position(anchor.py_arc_str())

    def __contains__(self, other) -> bool:
        return self.contains(other)

    def __str__(self) -> str:
        return self.__arc_str.to_string()

    def __getitem__(self, idx) -> "ArcStr":
        if isinstance(idx, slice):
            assert slice.step is not None, "Step is not supported"
            return ArcStr(self.__arc_str.slice(idx.start, idx.stop))
        elif isinstance(idx, int):
            return ArcStr(self.__arc_str.slice(idx, idx + 1))
        else:
            raise IndexError(f"Invalid index type: {type(idx)}")

    def __len__(self) -> int:
        return self.__arc_str.char_count()

    def py_arc_str(self) -> PyArcStr:
        return self.__arc_str

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

MapOut = TypeVar("MapOut")
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

    def map(self, cb: Callable[[ArcStr], MapOut]) -> List[MapOut]:
        return self.__buffer.map(lambda x: cb(ArcStr(x)))

    def select(self, items: List[int]) -> "Buffer":
        buf = Buffer.__new__(Buffer)
        buf.__buffer = self.__buffer.select(items)
        return buf

class Regex:
    __slots__ = ["__regex"]

    def __init__(self, pattern: str):
        self.__regex = PyRegex(pattern)

    def find(self, context: ArcStr) -> Optional[ArcStr]:
        astr = self.__regex.find(context.py_arc_str())
        if astr is None:
            return None
        return ArcStr(astr)

functools.lru_cache(maxsize=1024)
def compile_regex(pattern: str) -> PyRegex:
    return PyRegex(pattern)
