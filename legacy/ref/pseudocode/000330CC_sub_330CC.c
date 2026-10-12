/* ea:   0x000330CC */
/* name: sub_330CC */
/* size: 276 */
int __fastcall sub_330CC(int a1, HANDLE hFile)
{
  unsigned int i; // r4
  unsigned int j; // r4
  unsigned int Buffer; // [sp+4h] [bp-28h] BYREF
  DWORD NumberOfBytesRead; // [sp+8h] [bp-24h] BYREF
  int v9; // [sp+Ch] [bp-20h] BYREF
  int v10; // [sp+10h] [bp-1Ch] BYREF
  int v11; // [sp+14h] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  v10 = 0;
  sub_331E0(a1 + 16, Buffer, &v10);
  for ( i = 0; i < Buffer; ++i )
  {
    ReadFile(hFile, &v9, 4u, &NumberOfBytesRead, nullptr);
    *(_DWORD *)(*(_DWORD *)(a1 + 16) + 4 * i) = v9;
  }
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  v11 = 0;
  sub_331E0(a1 + 28, Buffer, &v11);
  for ( j = 0; j < Buffer; ++j )
  {
    ReadFile(hFile, &v9, 4u, &NumberOfBytesRead, nullptr);
    *(_DWORD *)(*(_DWORD *)(a1 + 28) + 4 * j) = v9;
  }
  return 1;
}

