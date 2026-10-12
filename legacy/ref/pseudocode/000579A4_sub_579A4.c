/* ea:   0x000579A4 */
/* name: sub_579A4 */
/* size: 1144 */
int __fastcall sub_579A4(int a1)
{
  int v2; // r5
  int v3; // r1
  int (*v4)(); // r3
  int v5; // r2
  int v6; // r0
  int v7; // r2
  int v8; // r4
  int v9; // r0
  _DWORD *v10; // r5
  int v11; // r1
  int (*v12)(); // r3
  int v13; // r1
  int v14; // r2
  int v15; // r0
  int v16; // r1
  int v17; // r2
  int v18; // r4
  int v19; // r1
  int v20; // r5
  int v21; // r1
  int (*v22)(); // r3
  int v23; // r2
  int v24; // r0
  int v25; // r2
  int v26; // r4
  int v27; // r0
  _DWORD *v28; // r5
  int v29; // r1
  int (*v30)(); // r3
  int v31; // r1
  int v32; // r2
  int v33; // r0
  int v34; // r1
  int v35; // r2
  int v36; // r4
  int v37; // r1
  int v38; // r1
  bool v39; // nf
  int result; // r0
  int v41; // [sp+Ch] [bp-A4h] BYREF
  int v42; // [sp+10h] [bp-A0h]
  int v43; // [sp+14h] [bp-9Ch]
  int v44; // [sp+18h] [bp-98h]
  _DWORD v45[4]; // [sp+1Ch] [bp-94h] BYREF
  _DWORD v46[4]; // [sp+2Ch] [bp-84h] BYREF
  _DWORD v47[4]; // [sp+3Ch] [bp-74h] BYREF
  _DWORD v48[4]; // [sp+4Ch] [bp-64h] BYREF
  _DWORD v49[4]; // [sp+5Ch] [bp-54h] BYREF
  _DWORD v50[4]; // [sp+6Ch] [bp-44h] BYREF
  _DWORD v51[13]; // [sp+7Ch] [bp-34h] BYREF

  sub_443FC((_DWORD *)a1);
  v2 = *(_DWORD *)(a1 + 1064);
  if ( v2 == 0 )
    __debugbreak();
  v3 = *(_DWORD *)(*(_DWORD *)(a1 + 1064) + 12);
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
  v41 = 0;
  v42 = v6;
  v43 = 0;
  v44 = v5;
  if ( v2 == 0 )
    __debugbreak();
  v7 = *(_DWORD *)(v2 + 8);
  if ( v2 == 0 )
    __debugbreak();
  v8 = *(_DWORD *)(a1 + 4);
  v45[0] = 8;
  v9 = *(_DWORD *)(v2 + 4);
  v45[2] = 7;
  v45[1] = v9 + 8;
  v45[3] = v7 + 7;
  (*(void (__fastcall **)(int, _DWORD *, int, int *, int (*)(), int *, int))(*(_DWORD *)v8 + 20))(
    v8,
    v45,
    v2,
    &v41,
    v4,
    &dword_81230,
    1);
  v10 = *(_DWORD **)(a1 + 1132);
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
  v13 = *v10;
  if ( *v10 == 0 )
    __debugbreak();
  v14 = *(_DWORD *)(*v10 + 8);
  if ( v13 == 0 )
    __debugbreak();
  v15 = *(_DWORD *)(v13 + 4);
  v46[0] = 0;
  v46[1] = v15;
  v46[2] = 0;
  v46[3] = v14;
  v16 = *v10;
  if ( *v10 == 0 )
    __debugbreak();
  v17 = *(_DWORD *)(*v10 + 8);
  if ( v16 == 0 )
    __debugbreak();
  v47[0] = 8;
  v18 = *(_DWORD *)(a1 + 4);
  v19 = *(_DWORD *)(v16 + 4) + 8;
  v47[2] = 7;
  v47[1] = v19;
  v47[3] = v17 + 7;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int, int, int, int, int))(*(_DWORD *)v18 + 20))(
    v18,
    v47,
    *v10,
    v46,
    v12,
    &dword_81230,
    1,
    v41,
    v42,
    v43,
    v44);
  v20 = *(_DWORD *)(a1 + 1076);
  if ( v20 == 0 )
    __debugbreak();
  v21 = *(_DWORD *)(*(_DWORD *)(a1 + 1076) + 12);
  if ( v21 == 16 )
  {
    v22 = sub_5BC14;
  }
  else if ( v21 == 8 )
  {
    v22 = sub_5D9C4;
  }
  else
  {
    v22 = nullptr;
  }
  *(_DWORD *)(a1 + 488) = v22;
  if ( v20 == 0 )
    __debugbreak();
  v23 = *(_DWORD *)(v20 + 8);
  if ( v20 == 0 )
    __debugbreak();
  v24 = *(_DWORD *)(v20 + 4);
  v48[0] = 0;
  v48[1] = v24;
  v48[2] = 0;
  v48[3] = v23;
  if ( v20 == 0 )
    __debugbreak();
  v25 = *(_DWORD *)(v20 + 8);
  if ( v20 == 0 )
    __debugbreak();
  v26 = *(_DWORD *)(a1 + 4);
  v49[0] = 109;
  v27 = *(_DWORD *)(v20 + 4);
  v49[2] = 151;
  v49[1] = v27 + 109;
  v49[3] = v25 + 151;
  (*(void (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v26 + 20))(
    v26,
    v49,
    v20,
    v48,
    v22,
    &dword_81230,
    1);
  v28 = *(_DWORD **)(a1 + 1132);
  if ( *v28 == 0 )
    __debugbreak();
  v29 = *(_DWORD *)(*v28 + 12);
  if ( v29 == 16 )
  {
    v30 = sub_5BC14;
  }
  else if ( v29 == 8 )
  {
    v30 = sub_5D9C4;
  }
  else
  {
    v30 = nullptr;
  }
  *(_DWORD *)(a1 + 488) = v30;
  v31 = *v28;
  if ( *v28 == 0 )
    __debugbreak();
  v32 = *(_DWORD *)(*v28 + 8);
  if ( v31 == 0 )
    __debugbreak();
  v33 = *(_DWORD *)(v31 + 4);
  v50[0] = 0;
  v50[1] = v33;
  v50[2] = 0;
  v50[3] = v32;
  v34 = *v28;
  if ( *v28 == 0 )
    __debugbreak();
  v35 = *(_DWORD *)(*v28 + 8);
  if ( v34 == 0 )
    __debugbreak();
  v51[0] = 109;
  v36 = *(_DWORD *)(a1 + 4);
  v37 = *(_DWORD *)(v34 + 4) + 109;
  v51[2] = 151;
  v51[1] = v37;
  v51[3] = v35 + 159;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v36 + 20))(
    v36,
    v51,
    *v28,
    v50,
    v30,
    &dword_81230,
    1);
  if ( *(_DWORD *)(a1 + 1020) != 0 )
  {
    v38 = *(_DWORD *)(a1 + 1028) + 1;
    v39 = *(_DWORD *)(a1 + 1028) - 47 < 0;
    *(_DWORD *)(a1 + 1028) = v38;
    if ( !(v39 ^ __OFSUB__(v38, 48) | (v38 == 48)) )
    {
      *(_DWORD *)(a1 + 1020) = 0;
      *(_DWORD *)(a1 + 1028) = 0;
    }
    return sub_573E4(a1);
  }
  else
  {
    sub_44828((_DWORD *)a1, *(unsigned __int16 *)(a1 + 1084), 100, 46);
    sub_44828((_DWORD *)a1, *(unsigned __int16 *)(a1 + 1086), 148, 46);
    sub_44828((_DWORD *)a1, *(unsigned __int16 *)(a1 + 1090), 38, 193);
    sub_44828((_DWORD *)a1, *(unsigned __int16 *)(a1 + 1092), 86, 193);
    if ( *(_DWORD *)(a1 + 1024) != 0 )
    {
      result = sub_573F4(a1);
      *(_DWORD *)(a1 + 1008) = 1;
    }
    else
    {
      return sub_575B8(a1);
    }
  }
  return result;
}

