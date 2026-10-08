/* ea:   0x0003FE84 */
/* name: sub_3FE84 */
/* size: 876 */
int __fastcall sub_3FE84(int a1, int a2, _DWORD *a3)
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
  int v15; // r0
  int v16; // r0
  int v17; // r0
  int v18; // r0
  int v19; // r0
  int v20; // r7
  void **v21; // r6
  int v22; // r0
  int v23; // r0
  int v24; // r5
  void *v25; // r0
  void *v26; // r4
  char *v28; // r4
  _WORD *v29; // [sp+4h] [bp-10Ch] BYREF
  int (**v30)(); // [sp+8h] [bp-108h] BYREF
  void *v31[5]; // [sp+Ch] [bp-104h] BYREF
  int (__fastcall **v32)(void *); // [sp+20h] [bp-F0h] BYREF
  char v33; // [sp+24h] [bp-ECh]
  int v34; // [sp+28h] [bp-E8h]
  _DWORD v35[12]; // [sp+38h] [bp-D8h] BYREF
  void *v36[3]; // [sp+68h] [bp-A8h] BYREF
  void *v37; // [sp+74h] [bp-9Ch] BYREF
  void *v38; // [sp+80h] [bp-90h] BYREF
  void *v39; // [sp+8Ch] [bp-84h] BYREF
  void *v40; // [sp+98h] [bp-78h] BYREF
  void *v41; // [sp+A4h] [bp-6Ch] BYREF
  void *v42; // [sp+B0h] [bp-60h] BYREF
  void *v43; // [sp+BCh] [bp-54h] BYREF
  void *v44; // [sp+C8h] [bp-48h] BYREF
  void *v45; // [sp+D4h] [bp-3Ch] BYREF
  void *v46; // [sp+E0h] [bp-30h] BYREF
  char v47; // [sp+ECh] [bp-24h] BYREF

  sub_1E61C((int)v35);
  sub_1DCEC((int)&v32);
  v32 = &off_6C938;
  sub_19490((int)v31);
  v6 = 11;
  v31[3] = nullptr;
  v30 = &off_6C90C;
  v7 = v36;
  for ( i = 11; i != 0; --i )
  {
    sub_1333C(v7);
    v7 += 3;
  }
  if ( (unsigned __int8)sub_63BE8(v35, a2) != 0 )
  {
    v9 = sub_12F04(aTargetKnifePcx);
    sub_1318C(v36, (char *)aTargetKnifePcx, (int)&aTargetKnifePcx[v9]);
    v10 = sub_12F04(aFaceKatharinaP);
    sub_1318C(&v37, (char *)aFaceKatharinaP, (int)&aFaceKatharinaP[v10]);
    v11 = sub_12F04(aFaceDrakePcx);
    sub_1318C(&v38, (char *)aFaceDrakePcx, (int)&aFaceDrakePcx[v11]);
    v12 = sub_12F04(aFaceKiddPcx);
    sub_1318C(&v39, (char *)aFaceKiddPcx, (int)&aFaceKiddPcx[v12]);
    v13 = sub_12F04(aFaceTichyPcx);
    sub_1318C(&v40, (char *)aFaceTichyPcx, (int)&aFaceTichyPcx[v13]);
    v14 = sub_12F04(aFacePandaPcx);
    sub_1318C(&v41, (char *)aFacePandaPcx, (int)&aFacePandaPcx[v14]);
    v15 = sub_12F04(aPlayerKatharin);
    sub_1318C(&v42, (char *)aPlayerKatharin, (int)&aPlayerKatharin[v15]);
    v16 = sub_12F04(aPlayerDrakePcx);
    sub_1318C(&v43, (char *)aPlayerDrakePcx, (int)&aPlayerDrakePcx[v16]);
    v17 = sub_12F04(aPlayerKiddPcx);
    sub_1318C(&v44, (char *)aPlayerKiddPcx, (int)&aPlayerKiddPcx[v17]);
    v18 = sub_12F04(aPlayerTichyPcx);
    sub_1318C(&v45, (char *)aPlayerTichyPcx, (int)&aPlayerTichyPcx[v18]);
    v19 = sub_12F04(aPlayerPandaPcx);
    sub_1318C(&v46, (char *)aPlayerPandaPcx, (int)&aPlayerPandaPcx[v19]);
    sub_1DDA4(a1 + 32, 0xBu);
    v20 = 0;
    v21 = v36;
    while ( 1 )
    {
      v29 = *v21;
      v22 = sub_1E5CC(&v29);
      v23 = sub_1E84C((int)v35, v22);
      v24 = v23;
      if ( v23 == 0 )
        __debugbreak();
      v25 = operator new(*(_DWORD *)(v23 + 4));
      v26 = v25;
      if ( v25 == nullptr )
        break;
      sub_63CE8((int)v35, v25);
      v34 = 16;
      v33 = 1;
      *(_DWORD *)(v20 + *a3) = sub_645B4(&v32, v26, *(_DWORD *)(v24 + 4));
      operator delete(v26);
      if ( *(_DWORD *)(v20 + *a3) == 0 )
      {
        sub_11634((int)v36, 12, 11, sub_12EA0);
        v30 = &off_6C90C;
        sub_194B4(v31);
        sub_1EA14(&v32);
        sub_1E798(v35);
        return 0;
      }
      v20 += 4;
      v21 += 3;
      if ( v20 >= 44 )
        goto LABEL_10;
    }
    v28 = &v47;
    do
    {
      v28 -= 12;
      sub_12EA0((int)v28);
      --v6;
    }
    while ( v6 != 0 );
    v30 = &off_6C90C;
    sub_194B4(v31);
    v32 = &off_6C910;
    sub_1E798(v35);
    return 0;
  }
  else
  {
LABEL_10:
    sub_11634((int)v36, 12, 11, sub_12EA0);
    sub_199D8(&v30);
    sub_1EA14(&v32);
    sub_1E798(v35);
    return 1;
  }
}

