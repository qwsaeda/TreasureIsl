/* ea:   0x00069FB0 */
/* name: start */
/* size: 112 */
void __fastcall __noreturn start(HINSTANCE a1, HINSTANCE a2, WCHAR *a3, int a4)
{
  int v8; // [sp+0h] [bp-20h]

  cinit();
  v8 = WinMain(a1, a2, a3, a4);
  exit(v8);
}

