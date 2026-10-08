/* ea:   0x00056EE4 */
/* name: sub_56EE4 */
/* size: 128 */
int __fastcall sub_56EE4(int a1, void *a2)
{
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( a2 == (void *)-1 )
    return 0;
  sub_52514(a1, a2);
  WriteFile(a2, (LPCVOID)(a1 + 208), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 212), 2u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 216), 4u, &NumberOfBytesWritten, nullptr);
  return 1;
}

