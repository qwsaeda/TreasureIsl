/* ea:   0x00052F7C */
/* name: sub_52F7C */
/* size: 184 */
int __fastcall sub_52F7C(_DWORD *a1, HANDLE hFile)
{
  _DWORD *i; // r4
  _BYTE v6[4]; // [sp+4h] [bp-24h] BYREF
  int Buffer; // [sp+8h] [bp-20h] BYREF
  DWORD NumberOfBytesWritten; // [sp+Ch] [bp-1Ch] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  Buffer = a1[1];
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  for ( i = *(_DWORD **)(*a1 + 8); i != (_DWORD *)*a1; i = sub_16A30(i) )
  {
    v6[0] = *((_BYTE *)i + 16);
    WriteFile(hFile, v6, 1u, &NumberOfBytesWritten, nullptr);
    sub_52EB4((char *)i + 18, hFile);
  }
  return 1;
}

