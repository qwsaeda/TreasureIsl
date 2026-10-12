/* ea:   0x0001B588 */
/* name: sub_1B588 */
/* size: 668 */
int __fastcall sub_1B588(int a1)
{
  int v2; // r5
  int v3; // r1
  int (*v4)(); // r3
  int v5; // r2
  int v6; // r0
  int v7; // r2
  int v8; // r0
  int v9; // r4
  _DWORD *v10; // r7
  int v11; // r1
  int (*v12)(); // r6
  int v13; // r0
  int v14; // r1
  int v15; // r0
  int v16; // r5
  int v17; // r2
  int v18; // r4
  int v20; // [sp+Ch] [bp-64h] BYREF
  int v21; // [sp+10h] [bp-60h]
  int v22; // [sp+14h] [bp-5Ch]
  int v23; // [sp+18h] [bp-58h]
  _DWORD v24[4]; // [sp+1Ch] [bp-54h] BYREF
  _DWORD v25[4]; // [sp+2Ch] [bp-44h] BYREF
  _DWORD v26[13]; // [sp+3Ch] [bp-34h] BYREF

  sub_443FC(a1);
  v2 = *(_DWORD *)(a1 + 1028);
  if ( v2 == 0 )
    __debugbreak();
  v3 = *(_DWORD *)(*(_DWORD *)(a1 + 1028) + 12);
  if ( v3 == 16 )
  {
    v4 = sub_5BC14;
  }
  else if ( v3 == 8 )
  {
    v4 = sub_5D9C4;
  }
  else
  {
    v4 = nullptr;
  }
  *(_DWORD *)(a1 + 488) = v4;
  if ( v2 == 0 )
    __debugbreak();
  v5 = *(_DWORD *)(v2 + 8);
  if ( v2 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(v2 + 4);
  v20 = 0;
  v21 = v6;
  v22 = 0;
  v23 = v5;
  if ( v2 == 0 )
    __debugbreak();
  v7 = *(_DWORD *)(v2 + 8);
  if ( v2 == 0 )
    __debugbreak();
  v24[0] = 8;
  v8 = *(_DWORD *)(v2 + 4);
  v24[2] = 7;
  v9 = *(_DWORD *)(a1 + 4);
  v24[1] = v8 + 8;
  v24[3] = v7 + 7;
  (*(void (__fastcall **)(int, _DWORD *, int, int *, int (*)(), int *, int))(*(_DWORD *)v9 + 20))(
    v9,
    v24,
    v2,
    &v20,
    v4,
    &dword_81230,
    1);
  v10 = *(_DWORD **)(a1 + 1112);
  if ( *v10 == 0 )
    __debugbreak();
  v11 = *(_DWORD *)(*v10 + 12);
  if ( v11 == 16 )
  {
    v12 = sub_5BC14;
  }
  else if ( v11 == 8 )
  {
    v12 = sub_5D9C4;
  }
  else
  {
    v12 = nullptr;
  }
  *(_DWORD *)(a1 + 488) = v12;
  v13 = v10[9];
  v14 = v10[10];
  v25[0] = 0;
  v25[1] = v13;
  v25[2] = 0;
  v25[3] = v14;
  v15 = v10[10];
  v16 = v10[4];
  v17 = v10[3] + v10[9];
  v26[0] = v10[3];
  v18 = *(_DWORD *)(a1 + 4);
  v26[3] = v16 + v15;
  v26[1] = v17;
  v26[2] = v16;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int, int, int, int, int))(*(_DWORD *)v18 + 20))(
    v18,
    v26,
    *v10,
    v25,
    v12,
    &dword_81230,
    1,
    v20,
    v21,
    v22,
    v23);
  sub_1B4E8((_DWORD *)a1);
  if ( *(_DWORD *)(a1 + 1020) == 0 )
  {
    sub_44714(a1, *(_DWORD *)(a1 + 1032), *(_DWORD *)(a1 + 1064) - 35, *(_DWORD *)(a1 + 1068) - 4);
    sub_44828(a1, *(_DWORD *)(a1 + 1040));
  }
  sub_44714(a1, *(unsigned __int16 *)(a1 + 1036), *(_DWORD *)(a1 + 1056) - 35, *(_DWORD *)(a1 + 1060) - 4);
  sub_44828(a1, *(unsigned __int16 *)(a1 + 1044));
  sub_44714(a1, *(unsigned __int16 *)(a1 + 1038), *(_DWORD *)(a1 + 1072) - 35, *(_DWORD *)(a1 + 1076) - 4);
  return sub_44828(a1, *(unsigned __int16 *)(a1 + 1046));
}

