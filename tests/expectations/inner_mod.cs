using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
struct Foo {
  float x;
};

[DllImport(DLL)]
static extern void root(Foo a);
}
