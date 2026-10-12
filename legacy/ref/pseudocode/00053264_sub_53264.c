/* ea:   0x00053264 */
/* name: sub_53264 */
/* size: 212 */
int __fastcall sub_53264(_DWORD *a1, HANDLE hFile)
{
  unsigned int i; // r6
  _DWORD *v5; // r0
  char v7[4]; // [sp+4h] [bp-24h] BYREF
  int v8; // [sp+8h] [bp-20h] BYREF
  unsigned int Buffer; // [sp+Ch] [bp-1Ch] BYREF
  DWORD NumberOfBytesRead; // [sp+10h] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  v8 = 0;
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  for ( i = 0; i < Buffer; *v5 = v8 )
  {
    ReadFile(hFile, v7, 1u, &NumberOfBytesRead, nullptr);
    sub_5319C((char *)&v8, hFile);
    v5 = (_DWORD *)sub_308D4(a1, v7);
    ++i;
  }
  return 1;
}

