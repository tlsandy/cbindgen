using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
const char DELIMITER = ':';

const int FOO = 10;

const char HEART = '\u2764';

const char LEFTCURLY = '{';

const sbyte NEG_ONE = -1;

const char NEWLINE = '\n';

const sbyte POS_ONE = 1;

const char QUOTE = '\'';

const char TAB = '\t';

const float ZOM = 3.14f;

struct Foo {
  [MarshalAs(UnmanagedType.ByValArray, SizeConst=FOO)] readonly int[] x;
};

[DllImport(DLL)]
static extern void root(Foo x);
}
