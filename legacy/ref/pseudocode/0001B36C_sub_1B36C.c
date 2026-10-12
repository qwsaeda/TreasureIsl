/* ea:   0x0001B36C */
/* name: sub_1B36C */
/* size: 172 */
int __fastcall sub_1B36C(int a1, HANDLE hFile)
{
  DWORD NumberOfBytesRead; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, (LPVOID)(a1 + 4), 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 8), 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 12), 2u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 14), 2u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 16), 2u, &NumberOfBytesRead, nullptr);
  return 1;
}

