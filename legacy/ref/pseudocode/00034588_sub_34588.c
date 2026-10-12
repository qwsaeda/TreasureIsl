/* ea:   0x00034588 */
/* name: sub_34588 */
/* size: 352 */
int __fastcall sub_34588(_DWORD *a1, HANDLE hFile)
{
  int v4; // t1
  int v5; // r0
  unsigned int i; // r4
  int *v7; // t2
  int v8; // r2
  unsigned int v9; // r5
  int v10; // r4
  int v11; // r0
  int v12; // r2
  int v13; // r3
  int v14; // r0
  unsigned int Buffer; // [sp+4h] [bp-34h] BYREF
  DWORD NumberOfBytesWritten; // [sp+8h] [bp-30h] BYREF
  int v18; // [sp+Ch] [bp-2Ch] BYREF
  int v19; // [sp+10h] [bp-28h]
  int v20; // [sp+14h] [bp-24h] BYREF
  int v21; // [sp+18h] [bp-20h]
  int v22; // [sp+1Ch] [bp-1Ch]

  if ( hFile == (HANDLE)-1 )
    return 0;
  v4 = a1[1];
  v18 = 0;
  v19 = 0;
  v5 = a1[2];
  v20 = 0;
  v21 = 0;
  LOBYTE(v22) = 0;
  Buffer = (v5 - v4) >> 3;
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  for ( i = 0; i < Buffer; ++i )
  {
    v7 = (int *)(a1[1] + 8 * i);
    v8 = v7[1];
    v18 = *v7;
    v19 = v8;
    WriteFile(hFile, &v18, 8u, &NumberOfBytesWritten, nullptr);
  }
  Buffer = (a1[9] - a1[8]) / 12;
  WriteFile(hFile, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
  v9 = 0;
  if ( Buffer != 0 )
  {
    v10 = 0;
    do
    {
      v11 = a1[8];
      v12 = *(_DWORD *)(v10 + v11);
      v13 = *(_DWORD *)(v10 + v11 + 4);
      v14 = *(_DWORD *)(v10 + v11 + 8);
      v20 = v12;
      v21 = v13;
      v22 = v14;
      WriteFile(hFile, &v20, 0xCu, &NumberOfBytesWritten, nullptr);
      ++v9;
      v10 += 12;
    }
    while ( v9 < Buffer );
  }
  return 1;
}

