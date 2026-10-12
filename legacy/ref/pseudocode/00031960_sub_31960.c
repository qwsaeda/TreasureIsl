/* ea:   0x00031960 */
/* name: sub_31960 */
/* size: 228 */
int __fastcall sub_31960(_DWORD *a1, HANDLE hFile)
{
  _BYTE *v4; // r0
  _BYTE Buffer[4]; // [sp+4h] [bp-20h] BYREF
  DWORD NumberOfBytesRead; // [sp+8h] [bp-1Ch] BYREF
  unsigned __int8 i; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, a1 + 1, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, a1 + 2, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, a1 + 7, 4u, &NumberOfBytesRead, nullptr);
  for ( i = 0; i < 0xAu; ++i )
  {
    ReadFile(hFile, Buffer, 1u, &NumberOfBytesRead, nullptr);
    v4 = (_BYTE *)sub_318D0(a1 + 8, &i);
    *v4 = Buffer[0];
  }
  ReadFile(hFile, a1 + 11, 4u, &NumberOfBytesRead, nullptr);
  return 1;
}

