/* ea:   0x0001B2C0 */
/* name: sub_1B2C0 */
/* size: 172 */
int __fastcall sub_1B2C0(int a1, HANDLE hFile)
{
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, (LPCVOID)(a1 + 4), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 8), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 12), 2u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 14), 2u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 16), 2u, &NumberOfBytesWritten, nullptr);
  return 1;
}

