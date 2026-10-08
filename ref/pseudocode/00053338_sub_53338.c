/* ea:   0x00053338 */
/* name: sub_53338 */
/* size: 164 */
int __fastcall sub_53338(int *a1, HANDLE hFile)
{
  unsigned int i; // r4
  _BYTE v6[3]; // [sp+5h] [bp-1Fh] BYREF
  unsigned int Buffer; // [sp+8h] [bp-1Ch] BYREF
  DWORD NumberOfBytesRead; // [sp+Ch] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  sub_5290C(a1, Buffer);
  for ( i = 0; i < Buffer; ++i )
  {
    ReadFile(hFile, v6, 1u, &NumberOfBytesRead, nullptr);
    *(_BYTE *)(*a1 + i) = v6[0];
  }
  return 1;
}

