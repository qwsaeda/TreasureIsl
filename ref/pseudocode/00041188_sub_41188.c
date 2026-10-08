/* ea:   0x00041188 */
/* name: sub_41188 */
/* size: 460 */
int __fastcall sub_41188(int a1)
{
  int v2; // r1
  _DWORD *v4; // r7
  int v5; // r2
  unsigned int v6; // r4
  int v7; // r5
  int v8; // r1
  unsigned int v9; // r3
  int v10; // r1
  int (*v11)(); // r6
  int v12; // r1
  unsigned int v13; // r5
  int v14; // r0
  int v15; // r4
  int v16; // r0
  int v17; // r4
  int v18; // r3
  int v19; // r0
  _DWORD v20[4]; // [sp+Ch] [bp-40h] BYREF
  _DWORD v21[12]; // [sp+1Ch] [bp-30h] BYREF

  v2 = *(_DWORD *)(a1 + 52);
  if ( v2 != 0 )
    return (*(int (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(*(_DWORD *)(a1 + 52));
  if ( *(_DWORD *)(a1 + 48) != 0 )
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 48) + 12))(*(_DWORD *)(a1 + 48));
  v4 = *(_DWORD **)(a1 + 32);
  v5 = *v4;
  if ( *v4 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(*v4 + 8);
  if ( v5 == 0 )
    __debugbreak();
  v7 = *(_DWORD *)(v5 + 4);
  if ( v5 == 0 )
    __debugbreak();
  v8 = *(_DWORD *)(a1 + 60);
  v9 = *(_DWORD *)(v5 + 8);
  v20[0] = 0;
  v20[1] = v7;
  v20[2] = (v9 >> 1) * (v8 / 8);
  v20[3] = (v6 >> 1) * (v8 / 8 + 1);
  if ( *v4 == 0 )
    __debugbreak();
  v10 = *(_DWORD *)(*v4 + 12);
  if ( v10 == 16 )
  {
    v11 = sub_5BC14;
  }
  else if ( v10 == 8 )
  {
    v11 = sub_5D9C4;
  }
  else
  {
    v11 = nullptr;
  }
  *(_DWORD *)(a1 + 12) = v11;
  v12 = *v4;
  if ( *v4 == 0 )
    __debugbreak();
  v13 = *(_DWORD *)(*v4 + 8);
  if ( v12 == 0 )
    __debugbreak();
  v14 = *(_DWORD *)(v12 + 4);
  v21[1] = 105;
  v15 = 105 - v14;
  v16 = *(_DWORD *)(a1 + 56);
  v21[0] = v15;
  v17 = *(_DWORD *)(a1 + 8);
  v21[3] = 20 * v16 + (v13 >> 1) + 77;
  v21[2] = 20 * v16 + 77;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v17 + 20))(
    v17,
    v21,
    *v4,
    v20,
    v11,
    &dword_81230,
    1);
  v18 = *(_DWORD *)(a1 + 60);
  if ( v18 >= 15 )
  {
    v19 = *(_DWORD *)(a1 + 4);
    *(_DWORD *)(a1 + 60) = 0;
    return sub_3DF94(v19);
  }
  else
  {
    *(_DWORD *)(a1 + 60) = v18 + 1;
    return sub_3DF94(*(_DWORD *)(a1 + 4));
  }
}

