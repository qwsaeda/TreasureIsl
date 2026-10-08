/* ea:   0x00044C70 */
/* name: sub_44C70 */
/* size: 264 */
int __fastcall sub_44C70(int a1, HANDLE hFile)
{
  int i; // r4
  _BYTE Buffer[4]; // [sp+4h] [bp-20h] BYREF
  DWORD NumberOfBytesWritten; // [sp+8h] [bp-1Ch] BYREF
  int v8; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, (LPCVOID)(a1 + 22), 2u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 24), 2u, &NumberOfBytesWritten, nullptr);
  Buffer[0] = *(_BYTE *)(a1 + 40);
  WriteFile(hFile, Buffer, 1u, &NumberOfBytesWritten, nullptr);
  Buffer[0] = *(_BYTE *)(a1 + 41);
  WriteFile(hFile, Buffer, 1u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 49), 1u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 50), 1u, &NumberOfBytesWritten, nullptr);
  for ( i = 0; i < 20; i += 4 )
  {
    v8 = *(_DWORD *)(i + *(_DWORD *)(a1 + 28));
    WriteFile(hFile, &v8, 4u, &NumberOfBytesWritten, nullptr);
  }
  return 1;
}

