/* ea:   0x00044D78 */
/* name: sub_44D78 */
/* size: 544 */
int __fastcall sub_44D78(int a1, HANDLE hFile, int a3)
{
  int i; // r4
  int v7; // r1
  int v8; // r0
  bool v9; // zf
  int v10; // r0
  int v11; // r1
  int v12; // r5
  int v13; // r3
  _DWORD *v14; // r0
  _DWORD *v15; // r0
  int v17; // r0
  _BYTE Buffer[4]; // [sp+4h] [bp-40h] BYREF
  DWORD NumberOfBytesRead; // [sp+8h] [bp-3Ch] BYREF
  int v20; // [sp+Ch] [bp-38h] BYREF
  int v21; // [sp+10h] [bp-34h] BYREF
  int v22; // [sp+14h] [bp-30h] BYREF
  int v23; // [sp+18h] [bp-2Ch]
  int v24; // [sp+1Ch] [bp-28h]
  int v25; // [sp+20h] [bp-24h]
  int v26; // [sp+24h] [bp-20h]
  int v27; // [sp+28h] [bp-1Ch]

  if ( hFile == (HANDLE)-1 )
    return 0;
  ReadFile(hFile, (LPVOID)(a1 + 22), 2u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 24), 2u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, Buffer, 1u, &NumberOfBytesRead, nullptr);
  *(_BYTE *)(a1 + 40) = Buffer[0];
  ReadFile(hFile, Buffer, 1u, &NumberOfBytesRead, nullptr);
  *(_BYTE *)(a1 + 41) = Buffer[0];
  ReadFile(hFile, (LPVOID)(a1 + 49), 1u, &NumberOfBytesRead, nullptr);
  ReadFile(hFile, (LPVOID)(a1 + 50), 1u, &NumberOfBytesRead, nullptr);
  if ( *(_BYTE *)(a1 + 50) != 0 )
    (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(a1 + 4) + 40) + 32))(
      *(_DWORD *)(*(_DWORD *)(a1 + 4) + 40),
      *(_DWORD *)(*(_DWORD *)(a3 + 32) + 4 * *(unsigned __int8 *)(a1 + 50) + 60));
  for ( i = 0; i < 20; i += 4 )
  {
    ReadFile(hFile, &v20, 4u, &NumberOfBytesRead, nullptr);
    *(_DWORD *)(i + *(_DWORD *)(a1 + 28)) = v20;
  }
  v7 = *(unsigned __int8 *)(a1 + 49);
  if ( *(_BYTE *)(a1 + 49) != 0 )
  {
    v8 = *(_DWORD *)(a1 + 44);
    v22 = 0;
    v9 = v8 == 1;
    v10 = *(_DWORD *)(a3 + 32);
    v23 = 0;
    v11 = v10 + 4 * v7;
    v24 = 0;
    v25 = 0;
    v26 = 0;
    v27 = 0;
    if ( v9 )
      v12 = *(_DWORD *)(v11 + 8);
    else
      v12 = *(_DWORD *)(v11 + 24);
    if ( v12 == 0 )
      __debugbreak();
    v13 = *(_DWORD *)(a1 + 16);
    v23 = *(_DWORD *)(a1 + 12) - *(_DWORD *)(v12 + 4) + 64;
    v24 = v13 - *(_DWORD *)(v12 + 8) + 32;
    v14 = operator new(0x14u);
    if ( v14 != nullptr )
      v15 = sub_430AC(v14);
    else
      v15 = nullptr;
    *(_DWORD *)(a1 + 8) = v15;
    if ( v15 == nullptr )
      return 0;
    sub_11558(a3 + 64, &v21, *(_DWORD *)(a3 + 64), (_DWORD *)(a1 + 8));
    v17 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a3 + 64) + 4) + 8);
    (*(void (__fastcall **)(int, int, int *, int))(*(_DWORD *)v17 + 8))(v17, v12, &v22, 1);
  }
  return 1;
}

