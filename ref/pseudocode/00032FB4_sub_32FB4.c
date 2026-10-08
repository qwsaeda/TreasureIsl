/* ea:   0x00032FB4 */
/* name: sub_32FB4 */
/* size: 280 */
int __fastcall sub_32FB4(_DWORD *a1, HANDLE hFile)
{
  unsigned int i; // r4
  unsigned int j; // r4
  unsigned int Buffer; // [sp+4h] [bp-20h] BYREF
  DWORD NumberOfBytesWritten; // [sp+8h] [bp-1Ch] BYREF
  _DWORD v9[6]; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  Buffer = (a1[5] - a1[4]) >> 2;
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  for ( i = 0; i < Buffer; ++i )
  {
    v9[0] = *(_DWORD *)(a1[4] + 4 * i);
    WriteFile(hFile, v9, 4u, &NumberOfBytesWritten, nullptr);
  }
  Buffer = (a1[8] - a1[7]) >> 2;
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  for ( j = 0; j < Buffer; ++j )
  {
    v9[0] = *(_DWORD *)(a1[7] + 4 * j);
    WriteFile(hFile, v9, 4u, &NumberOfBytesWritten, nullptr);
  }
  return 1;
}

