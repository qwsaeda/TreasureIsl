/* ea:   0x00053034 */
/* name: sub_53034 */
/* size: 160 */
int __fastcall sub_53034(_DWORD *a1, HANDLE hFile)
{
  unsigned int i; // r4
  _BYTE v6[4]; // [sp+4h] [bp-20h] BYREF
  unsigned int Buffer; // [sp+8h] [bp-1Ch] BYREF
  DWORD NumberOfBytesWritten; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  Buffer = a1[1] - *a1;
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  for ( i = 0; i < Buffer; ++i )
  {
    v6[0] = *(_BYTE *)(*a1 + i);
    WriteFile(hFile, v6, 1u, &NumberOfBytesWritten, nullptr);
  }
  return 1;
}

