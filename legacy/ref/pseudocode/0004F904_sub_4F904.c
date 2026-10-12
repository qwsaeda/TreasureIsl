/* ea:   0x0004F904 */
/* name: sub_4F904 */
/* size: 680 */
int __fastcall sub_4F904(int a1, int a2, int a3)
{
  int v4; // r0
  int v5; // r0
  int v6; // r0
  int v7; // r5
  void *v8; // r0
  void *v9; // r4
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r5
  void *v15; // r0
  void *v16; // r4
  _WORD *v17; // [sp+4h] [bp-90h] BYREF
  _WORD *v18; // [sp+8h] [bp-8Ch] BYREF
  void *v19[4]; // [sp+10h] [bp-84h] BYREF
  void *v20[4]; // [sp+20h] [bp-74h] BYREF
  int (__fastcall **v21)(void *); // [sp+30h] [bp-64h] BYREF
  char v22; // [sp+34h] [bp-60h]
  int v23; // [sp+38h] [bp-5Ch]
  _DWORD v24[19]; // [sp+48h] [bp-4Ch] BYREF

  if ( *(_DWORD *)(a1 + 48) == 0 )
    return 0;
  if ( *(_DWORD *)(a1 + 52) == 0 )
    return 0;
  *(_DWORD *)(a1 + 4) = a2;
  if ( a2 == 0 )
    return 0;
  *(_DWORD *)(a1 + 8) = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v20);
  sub_1333C(v19);
  sub_1DEBC(a1 + 32, 3u);
  sub_1DDA4(a1 + 32, 3u);
  sub_1309C((int)v20);
  sub_162DC(v20, (char *)aDataGlpicsPak);
  sub_1E61C((int)v24);
  sub_1DCEC((int)&v21);
  v21 = &off_6C938;
  if ( (unsigned __int8)sub_63BE8(v24, v20[0]) == 0 )
    goto LABEL_16;
  v4 = sub_12F04(aTargetLocaterP);
  sub_1318C(v19, (char *)aTargetLocaterP, (int)&aTargetLocaterP[v4]);
  v17 = v19[0];
  v5 = sub_1E5CC(&v17);
  v6 = sub_1E84C((int)v24, v5);
  v7 = v6;
  if ( v6 == 0 )
    __debugbreak();
  v8 = operator new(*(_DWORD *)(v6 + 4));
  v9 = v8;
  if ( v8 == nullptr
    || (sub_63CE8((int)v24, v8),
        v22 = 1,
        v23 = 16,
        **(_DWORD **)(a1 + 32) = sub_645B4(&v21, v9, *(_DWORD *)(v7 + 4)),
        operator delete(v9),
        **(_DWORD **)(a1 + 32) == 0) )
  {
    v21 = &off_6C910;
    sub_1E798(v24);
    sub_12F48(v19);
    sub_12F48(v20);
    return 0;
  }
  v11 = sub_12F04(aTargetAreaPcx);
  sub_1318C(v19, (char *)aTargetAreaPcx, (int)&aTargetAreaPcx[v11]);
  v18 = v19[0];
  v12 = sub_1E5CC(&v18);
  v13 = sub_1E84C((int)v24, v12);
  v14 = v13;
  if ( v13 == 0 )
    __debugbreak();
  v15 = operator new(*(_DWORD *)(v13 + 4));
  v16 = v15;
  if ( v15 != nullptr
    && (sub_63CE8((int)v24, v15),
        v23 = 16,
        v22 = 1,
        *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4) = sub_645B4(&v21, v16, *(_DWORD *)(v14 + 4)),
        operator delete(v16),
        *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4) != 0) )
  {
LABEL_16:
    sub_1EA14(&v21);
    sub_1E798(v24);
    sub_12EA0((int)v19);
    sub_12EA0((int)v20);
    return 1;
  }
  else
  {
    sub_1EA14(&v21);
    sub_1E798(v24);
    sub_12EA0((int)v19);
    sub_12EA0((int)v20);
    return 0;
  }
}

