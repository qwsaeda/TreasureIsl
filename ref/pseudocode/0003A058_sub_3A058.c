/* ea:   0x0003A058 */
/* name: sub_3A058 */
/* size: 1528 */
int __fastcall sub_3A058(_DWORD *a1, int a2)
{
  int v4; // r7
  void **v5; // r4
  int i; // r5
  int v7; // r0
  int v8; // r0
  int v9; // r0
  int v10; // r0
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r5
  void *v15; // r0
  void *v16; // r4
  _BYTE *v17; // r4
  _BYTE *v18; // r4
  int v19; // r0
  int v20; // r0
  int v21; // r5
  void *v22; // r0
  void *v23; // r4
  _BYTE *v24; // r4
  _BYTE *v25; // r4
  int v26; // r0
  int v27; // r0
  int v28; // r5
  void *v29; // r0
  void *v30; // r4
  _BYTE *v31; // r4
  _BYTE *v32; // r4
  int v33; // r0
  int v34; // r0
  int v35; // r5
  void *v36; // r0
  void *v37; // r4
  _BYTE *v38; // r4
  _BYTE *v39; // r4
  int v41; // r0
  int v42; // r0
  int v43; // r5
  void *v44; // r0
  void *v45; // r4
  int (__fastcall **v46)(void *); // [sp+0h] [bp-B8h] BYREF
  char v47; // [sp+4h] [bp-B4h]
  int v48; // [sp+8h] [bp-B0h]
  _WORD *v49; // [sp+18h] [bp-A0h] BYREF
  _WORD *v50; // [sp+1Ch] [bp-9Ch] BYREF
  _WORD *v51; // [sp+20h] [bp-98h] BYREF
  _WORD *v52; // [sp+24h] [bp-94h] BYREF
  _WORD *v53; // [sp+28h] [bp-90h] BYREF
  _DWORD v54[12]; // [sp+30h] [bp-88h] BYREF
  void *v55[3]; // [sp+60h] [bp-58h] BYREF
  _WORD *v56; // [sp+6Ch] [bp-4Ch] BYREF
  _WORD *v57; // [sp+78h] [bp-40h] BYREF
  _WORD *v58; // [sp+84h] [bp-34h] BYREF
  _WORD *v59; // [sp+90h] [bp-28h] BYREF
  _BYTE v60[28]; // [sp+9Ch] [bp-1Ch] BYREF

  sub_1E61C((int)v54);
  sub_1DCEC((int)&v46);
  v4 = 5;
  v46 = &off_6C938;
  v5 = v55;
  for ( i = 5; i != 0; --i )
  {
    sub_1333C(v5);
    v5 += 3;
  }
  if ( (unsigned __int8)sub_63BE8(v54, a2) == 0 )
    goto LABEL_50;
  v7 = sub_12F04(aHelmStillPcx);
  sub_1318C(v55, (char *)aHelmStillPcx, (int)&aHelmStillPcx[v7]);
  v8 = sub_12F04(aHelmNumberPcx);
  sub_1318C((void **)&v56, (char *)aHelmNumberPcx, (int)&aHelmNumberPcx[v8]);
  v9 = sub_12F04(aHelmBasePcx);
  sub_1318C((void **)&v57, (char *)aHelmBasePcx, (int)&aHelmBasePcx[v9]);
  v10 = sub_12F04(aHelmPointerPcx);
  sub_1318C((void **)&v58, (char *)aHelmPointerPcx, (int)&aHelmPointerPcx[v10]);
  v11 = sub_12F04(aHelmMotionPcx);
  sub_1318C((void **)&v59, (char *)aHelmMotionPcx, (int)&aHelmMotionPcx[v11]);
  v52 = v55[0];
  v12 = sub_1E5CC(&v52);
  v13 = sub_1E84C((int)v54, v12);
  v14 = v13;
  if ( v13 == 0 )
    __debugbreak();
  v15 = operator new(*(_DWORD *)(v13 + 4));
  v16 = v15;
  if ( v15 == nullptr )
  {
    v17 = v60;
    do
    {
      v17 -= 12;
      sub_12EA0((int)v17);
      --v4;
    }
    while ( v4 != 0 );
LABEL_43:
    v46 = &off_6C910;
    sub_1E798(v54);
    return 0;
  }
  sub_63CE8((int)v54, v15);
  v47 = 1;
  v48 = 16;
  a1[14] = sub_645B4(&v46, v16, *(_DWORD *)(v14 + 4));
  operator delete(v16);
  if ( a1[14] == 0 )
  {
    v18 = v60;
    do
    {
      v18 -= 12;
      sub_12EA0((int)v18);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  v51 = v56;
  v19 = sub_1E5CC(&v51);
  v20 = sub_1E84C((int)v54, v19);
  v21 = v20;
  if ( v20 == 0 )
    __debugbreak();
  v22 = operator new(*(_DWORD *)(v20 + 4));
  v23 = v22;
  if ( v22 == nullptr )
  {
    v24 = v60;
    do
    {
      v24 -= 12;
      sub_12EA0((int)v24);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  sub_63CE8((int)v54, v22);
  v48 = 16;
  v47 = 1;
  a1[17] = sub_645B4(&v46, v23, *(_DWORD *)(v21 + 4));
  operator delete(v23);
  if ( a1[17] == 0 )
  {
    v25 = v60;
    do
    {
      v25 -= 12;
      sub_12EA0((int)v25);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  v53 = v57;
  v26 = sub_1E5CC(&v53);
  v27 = sub_1E84C((int)v54, v26);
  v28 = v27;
  if ( v27 == 0 )
    __debugbreak();
  v29 = operator new(*(_DWORD *)(v27 + 4));
  v30 = v29;
  if ( v29 == nullptr )
  {
    v31 = v60;
    do
    {
      v31 -= 12;
      sub_12EA0((int)v31);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  sub_63CE8((int)v54, v29);
  v48 = 16;
  v47 = 1;
  a1[13] = sub_645B4(&v46, v30, *(_DWORD *)(v28 + 4));
  operator delete(v30);
  if ( a1[13] == 0 )
  {
    v32 = v60;
    do
    {
      v32 -= 12;
      sub_12EA0((int)v32);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  v49 = v58;
  v33 = sub_1E5CC(&v49);
  v34 = sub_1E84C((int)v54, v33);
  v35 = v34;
  if ( v34 == 0 )
    __debugbreak();
  v36 = operator new(*(_DWORD *)(v34 + 4));
  v37 = v36;
  if ( v36 == nullptr )
  {
    v38 = v60;
    do
    {
      v38 -= 12;
      sub_12EA0((int)v38);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  sub_63CE8((int)v54, v36);
  v48 = 16;
  v47 = 1;
  a1[16] = sub_645B4(&v46, v37, *(_DWORD *)(v35 + 4));
  operator delete(v37);
  if ( a1[16] == 0 )
  {
    v39 = v60;
    do
    {
      v39 -= 12;
      sub_12EA0((int)v39);
      --v4;
    }
    while ( v4 != 0 );
    goto LABEL_43;
  }
  v50 = v59;
  v41 = sub_1E5CC(&v50);
  v42 = sub_1E84C((int)v54, v41);
  v43 = v42;
  if ( v42 == 0 )
    __debugbreak();
  v44 = operator new(*(_DWORD *)(v42 + 4));
  v45 = v44;
  if ( v44 == nullptr )
  {
    sub_11634((int)v55, 12, 5, sub_12EA0);
    v46 = &off_6C910;
    sub_1E798(v54);
    return 0;
  }
  sub_63CE8((int)v54, v44);
  v48 = 16;
  v47 = 1;
  a1[15] = sub_645B4(&v46, v45, *(_DWORD *)(v43 + 4));
  operator delete(v45);
  if ( a1[15] != 0 )
  {
LABEL_50:
    sub_11634((int)v55, 12, 5, sub_12EA0);
    sub_1EA14(&v46);
    sub_1E798(v54);
    return 1;
  }
  else
  {
    sub_11634((int)v55, 12, 5, sub_12EA0);
    sub_1EA14(&v46);
    sub_1E798(v54);
    return 0;
  }
}

