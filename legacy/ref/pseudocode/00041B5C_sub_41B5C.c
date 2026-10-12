/* ea:   0x00041B5C */
/* name: sub_41B5C */
/* size: 728 */
int __fastcall sub_41B5C(int a1, int a2, _DWORD *a3)
{
  int v6; // r9
  void **v7; // r4
  int i; // r5
  int v9; // r0
  int v10; // r0
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r0
  int v15; // r7
  void **v16; // r6
  int v17; // r0
  int v18; // r0
  int v19; // r5
  void *v20; // r0
  void *v21; // r4
  _DWORD *v23; // r4
  _DWORD *v24; // r4
  _WORD *v25; // [sp+4h] [bp-D0h] BYREF
  int (**v26)(); // [sp+8h] [bp-CCh] BYREF
  void *v27[5]; // [sp+Ch] [bp-C8h] BYREF
  int (__fastcall **v28)(void *); // [sp+20h] [bp-B4h] BYREF
  char v29; // [sp+24h] [bp-B0h]
  int v30; // [sp+28h] [bp-ACh]
  void *v31[3]; // [sp+38h] [bp-9Ch] BYREF
  void *v32; // [sp+44h] [bp-90h] BYREF
  void *v33; // [sp+50h] [bp-84h] BYREF
  void *v34; // [sp+5Ch] [bp-78h] BYREF
  void *v35; // [sp+68h] [bp-6Ch] BYREF
  void *v36; // [sp+74h] [bp-60h] BYREF
  _DWORD v37[21]; // [sp+80h] [bp-54h] BYREF

  sub_1E61C((int)v37);
  sub_1DCEC((int)&v28);
  v28 = &off_6C938;
  sub_19490((int)v27);
  v6 = 6;
  v27[3] = nullptr;
  v26 = &off_6C90C;
  v7 = v31;
  for ( i = 6; i != 0; --i )
  {
    sub_1333C(v7);
    v7 += 3;
  }
  if ( (unsigned __int8)sub_63BE8(v37, a2) != 0 )
  {
    v9 = sub_12F04(aTargetHeadPcx);
    sub_1318C(v31, (char *)aTargetHeadPcx, (int)&aTargetHeadPcx[v9]);
    v10 = sub_12F04(aPlayerKatharin);
    sub_1318C(&v32, (char *)aPlayerKatharin, (int)&aPlayerKatharin[v10]);
    v11 = sub_12F04(aPlayerDrakePcx);
    sub_1318C(&v33, (char *)aPlayerDrakePcx, (int)&aPlayerDrakePcx[v11]);
    v12 = sub_12F04(aPlayerKiddPcx);
    sub_1318C(&v34, (char *)aPlayerKiddPcx, (int)&aPlayerKiddPcx[v12]);
    v13 = sub_12F04(aPlayerTichyPcx);
    sub_1318C(&v35, (char *)aPlayerTichyPcx, (int)&aPlayerTichyPcx[v13]);
    v14 = sub_12F04(aPlayerPandaPcx);
    sub_1318C(&v36, (char *)aPlayerPandaPcx, (int)&aPlayerPandaPcx[v14]);
    sub_1DDA4(a1 + 32, 6u);
    v15 = 0;
    v16 = v31;
    while ( 1 )
    {
      v25 = *v16;
      v17 = sub_1E5CC(&v25);
      v18 = sub_1E84C((int)v37, v17);
      v19 = v18;
      if ( v18 == 0 )
        __debugbreak();
      v20 = operator new(*(_DWORD *)(v18 + 4));
      v21 = v20;
      if ( v20 == nullptr )
        break;
      sub_63CE8((int)v37, v20);
      v30 = 16;
      v29 = 1;
      *(_DWORD *)(v15 + *a3) = sub_645B4(&v28, v21, *(_DWORD *)(v19 + 4));
      operator delete(v21);
      if ( *(_DWORD *)(v15 + *a3) == 0 )
      {
        v24 = v37;
        do
        {
          v24 -= 3;
          sub_12EA0((int)v24);
          --v6;
        }
        while ( v6 != 0 );
        v26 = &off_6C90C;
        sub_194B4(v27);
        sub_1EA14(&v28);
        sub_1E798(v37);
        return 0;
      }
      v15 += 4;
      v16 += 3;
      if ( v15 >= 24 )
        goto LABEL_10;
    }
    v23 = v37;
    do
    {
      v23 -= 3;
      sub_12EA0((int)v23);
      --v6;
    }
    while ( v6 != 0 );
    v26 = &off_6C90C;
    sub_194B4(v27);
    v28 = &off_6C910;
    sub_1E798(v37);
    return 0;
  }
  else
  {
LABEL_10:
    sub_11634((int)v31, 12, 6, sub_12EA0);
    sub_199D8(&v26);
    sub_1EA14(&v28);
    sub_1E798(v37);
    return 1;
  }
}

