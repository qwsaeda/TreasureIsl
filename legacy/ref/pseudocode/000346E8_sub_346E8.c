/* ea:   0x000346E8 */
/* name: sub_346E8 */
/* size: 348 */
int __fastcall sub_346E8(int a1, HANDLE hFile)
{
  unsigned int i; // r4
  int v5; // r3
  int v6; // r1
  int v7; // r3
  unsigned int v8; // r6
  int v9; // r5
  int v10; // r3
  int v11; // r4
  int v12; // r1
  int v13; // r2
  int v14; // r4
  unsigned int Buffer; // [sp+4h] [bp-50h] BYREF
  DWORD NumberOfBytesRead; // [sp+8h] [bp-4Ch] BYREF
  int v18; // [sp+Ch] [bp-48h] BYREF
  int v19; // [sp+10h] [bp-44h]
  _DWORD v20[2]; // [sp+14h] [bp-40h] BYREF
  int v21; // [sp+1Ch] [bp-38h] BYREF
  int v22; // [sp+20h] [bp-34h]
  int v23; // [sp+24h] [bp-30h]
  _DWORD v24[2]; // [sp+2Ch] [bp-28h] BYREF
  char v25; // [sp+34h] [bp-20h]

  if ( hFile == (HANDLE)-1 )
    return 0;
  v18 = 0;
  v19 = 0;
  v21 = 0;
  v22 = 0;
  LOBYTE(v23) = 0;
  Buffer = 0;
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  v20[0] = 0;
  v20[1] = 0;
  sub_34844(a1 + 4, Buffer, v20);
  for ( i = 0; i < Buffer; *(_DWORD *)(v7 + 4) = v6 )
  {
    ReadFile(hFile, &v18, 8u, &NumberOfBytesRead, nullptr);
    v5 = *(_DWORD *)(a1 + 4);
    v6 = v19;
    *(_DWORD *)(v5 + 8 * i) = v18;
    v7 = v5 + 8 * i++;
  }
  ReadFile(hFile, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  v24[0] = 0;
  v24[1] = 0;
  v25 = 0;
  sub_34890(a1 + 32, Buffer, v24);
  v8 = 0;
  if ( Buffer != 0 )
  {
    v9 = 0;
    do
    {
      ReadFile(hFile, &v21, 0xCu, &NumberOfBytesRead, nullptr);
      v10 = *(_DWORD *)(a1 + 32);
      v11 = v9;
      v12 = v22;
      ++v8;
      v13 = v23;
      v9 += 12;
      *(_DWORD *)(v11 + v10) = v21;
      v14 = v11 + v10;
      *(_DWORD *)(v14 + 4) = v12;
      *(_DWORD *)(v14 + 8) = v13;
    }
    while ( v8 < Buffer );
  }
  return 1;
}

