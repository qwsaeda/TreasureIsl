/* ea:   0x0001CCE8 */
/* name: sub_1CCE8 */
/* size: 1068 */
int __fastcall sub_1CCE8(int a1, int a2, int a3)
{
  int v5; // r0
  int v6; // r0
  int v7; // r0
  int v8; // r5
  void *v9; // r0
  void *v10; // r4
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r5
  void *v15; // r0
  void *v16; // r4
  int v17; // r0
  int v18; // r0
  int v19; // r0
  int v20; // r5
  void *v21; // r0
  void *v22; // r4
  int v23; // r0
  int v24; // r0
  int v25; // r0
  int v26; // r5
  void *v27; // r0
  void *v28; // r4
  void *v29; // [sp+4h] [bp-98h] BYREF
  void *v30; // [sp+8h] [bp-94h] BYREF
  void *v31; // [sp+Ch] [bp-90h] BYREF
  void *v32; // [sp+10h] [bp-8Ch] BYREF
  void *v33[4]; // [sp+18h] [bp-84h] BYREF
  void *v34[4]; // [sp+28h] [bp-74h] BYREF
  int (__fastcall **v35)(void *); // [sp+38h] [bp-64h] BYREF
  char v36; // [sp+3Ch] [bp-60h]
  int v37; // [sp+40h] [bp-5Ch]
  _BYTE v38[76]; // [sp+50h] [bp-4Ch] BYREF

  if ( *(_DWORD *)(a1 + 48) == 0 )
    return 0;
  *(_DWORD *)(a1 + 4) = a2;
  if ( a2 == 0 )
    return 0;
  *(_DWORD *)(a1 + 8) = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v34);
  sub_1333C(v33);
  sub_1DEBC(a1 + 32, 4);
  sub_1DDA4(a1 + 32);
  sub_1309C((int)v34);
  sub_162DC(v34, (char *)aDataGlpicsPak);
  sub_1E61C(v38);
  sub_1DCEC(&v35);
  v35 = &off_6C938;
  if ( (unsigned __int8)sub_63BE8(v38, v34[0]) == 0 )
    goto LABEL_25;
  v5 = sub_12F04(aTargetLocaterP);
  sub_1318C(v33, (char *)aTargetLocaterP, (int)&aTargetLocaterP[v5]);
  v29 = v33[0];
  v6 = sub_1E5CC(&v29);
  v7 = sub_1E84C(v38, v6);
  v8 = v7;
  if ( v7 == 0 )
    __debugbreak();
  v9 = operator new(*(_DWORD *)(v7 + 4));
  v10 = v9;
  if ( v9 == nullptr )
    goto LABEL_17;
  sub_63CE8((int)v38, v9);
  v36 = 1;
  v37 = 16;
  **(_DWORD **)(a1 + 32) = sub_645B4(&v35, v10, *(_DWORD *)(v8 + 4));
  operator delete(v10);
  if ( **(_DWORD **)(a1 + 32) == 0 )
    goto LABEL_17;
  v11 = sub_12F04(aTargetAreaPcx);
  sub_1318C(v33, (char *)aTargetAreaPcx, (int)&aTargetAreaPcx[v11]);
  v30 = v33[0];
  v12 = sub_1E5CC(&v30);
  v13 = sub_1E84C(v38, v12);
  v14 = v13;
  if ( v13 == 0 )
    __debugbreak();
  v15 = operator new(*(_DWORD *)(v13 + 4));
  v16 = v15;
  if ( v15 == nullptr )
    goto LABEL_17;
  sub_63CE8((int)v38, v15);
  v37 = 16;
  v36 = 1;
  *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4) = sub_645B4(&v35, v16, *(_DWORD *)(v14 + 4));
  operator delete(v16);
  if ( *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4) == 0 )
    goto LABEL_17;
  v17 = sub_12F04(aNumberPcx);
  sub_1318C(v33, (char *)aNumberPcx, (int)&aNumberPcx[v17]);
  v31 = v33[0];
  v18 = sub_1E5CC(&v31);
  v19 = sub_1E84C(v38, v18);
  v20 = v19;
  if ( v19 == 0 )
    __debugbreak();
  v21 = operator new(*(_DWORD *)(v19 + 4));
  v22 = v21;
  if ( v21 == nullptr )
  {
LABEL_17:
    v35 = &off_6C910;
    sub_1E798(v38);
    sub_12F48(v33);
    sub_12F48(v34);
    return 0;
  }
  sub_63CE8((int)v38, v21);
  v37 = 16;
  v36 = 1;
  *(_DWORD *)(a1 + 16) = sub_645B4(&v35, v22, *(_DWORD *)(v20 + 4));
  operator delete(v22);
  if ( *(_DWORD *)(a1 + 16) == 0 )
  {
    v35 = &off_6C910;
    sub_1E798(v38);
    sub_12EA0((int)v33);
    sub_12EA0((int)v34);
    return 0;
  }
  v23 = sub_12F04(aLandinfoPcx);
  sub_1318C(v33, (char *)aLandinfoPcx, (int)&aLandinfoPcx[v23]);
  v32 = v33[0];
  v24 = sub_1E5CC(&v32);
  v25 = sub_1E84C(v38, v24);
  v26 = v25;
  if ( v25 == 0 )
    __debugbreak();
  v27 = operator new(*(_DWORD *)(v25 + 4));
  v28 = v27;
  if ( v27 != nullptr
    && (sub_63CE8((int)v38, v27),
        v37 = 16,
        v36 = 1,
        *(_DWORD *)(*(_DWORD *)(a1 + 32) + 12) = sub_645B4(&v35, v28, *(_DWORD *)(v26 + 4)),
        operator delete(v28),
        *(_DWORD *)(*(_DWORD *)(a1 + 32) + 12) != 0) )
  {
LABEL_25:
    *(_DWORD *)(a1 + 76) = -1;
    *(_DWORD *)(a1 + 80) = -1;
    sub_1EA14(&v35);
    sub_1E798(v38);
    sub_12EA0((int)v33);
    sub_12EA0((int)v34);
    return 1;
  }
  else
  {
    sub_1EA14(&v35);
    sub_1E798(v38);
    sub_12EA0((int)v33);
    sub_12EA0((int)v34);
    return 0;
  }
}

