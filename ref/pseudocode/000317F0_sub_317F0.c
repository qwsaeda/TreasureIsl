/* ea:   0x000317F0 */
/* name: sub_317F0 */
/* size: 224 */
int __fastcall sub_317F0(int a1, HANDLE hFile)
{
  _BYTE Buffer[4]; // [sp+4h] [bp-20h] BYREF
  DWORD NumberOfBytesWritten; // [sp+8h] [bp-1Ch] BYREF
  unsigned __int8 i; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, (LPCVOID)(a1 + 4), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 8), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, (LPCVOID)(a1 + 28), 4u, &NumberOfBytesWritten, nullptr);
  for ( i = 0; i < 0xAu; ++i )
  {
    Buffer[0] = *(_BYTE *)sub_318D0(a1 + 32, &i);
    WriteFile(hFile, Buffer, 1u, &NumberOfBytesWritten, nullptr);
  }
  WriteFile(hFile, (LPCVOID)(a1 + 44), 4u, &NumberOfBytesWritten, nullptr);
  return 1;
}

