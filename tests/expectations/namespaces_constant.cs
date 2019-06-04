using System.Runtime.InteropServices;

namespace constants {
namespace test {
static class Imports {
const string DLL = ".dll";
const int FOO = 10;

const float ZOM = 3.14f;

struct Foo {
  [MarshalAs(UnmanagedType.ByValArray, SizeConst=FOO)] readonly int[] x;
};

[DllImport(DLL)]
static extern void root(Foo x);
}

} // namespace test
} // namespace constants
