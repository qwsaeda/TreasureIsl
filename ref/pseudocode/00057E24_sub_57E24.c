/* ea:   0x00057E24 */
/* name: sub_57E24 */
/* size: 1464 */
void sub_57E24(_DWORD *a1, ...)
{
  _DWORD *v2; // r4
  _DWORD *v3; // r0
  _DWORD *v4; // r0
  _DWORD *v5; // r0
  int v6; // r3
  _DWORD *v7; // r1
  _DWORD *v8; // r4
  _DWORD *v9; // r0
  _DWORD *v10; // r0
  int v11; // r3
  _DWORD *v12; // r1
  _DWORD *v13; // r4
  _DWORD *v14; // r0
  _DWORD *v15; // r0
  int v16; // r3
  _DWORD *v17; // r1
  _DWORD *v18; // r4
  _DWORD *v19; // r0
  _DWORD *v20; // r0
  int v21; // r3
  _DWORD *v22; // r1
  _DWORD *v23; // r4
  _DWORD *v24; // r0
  _DWORD *v25; // r0
  int v26; // r3
  _DWORD *v27; // r1
  _DWORD *v28; // r4
  _DWORD *v29; // r0
  _DWORD *v30; // r0
  int v31; // r3
  _DWORD *v32; // r1
  _DWORD *v33; // r4
  _DWORD *v34; // r0
  _DWORD *v35; // r0
  int v36; // r3
  _DWORD *v37; // r1
  _DWORD *v38; // r4
  _DWORD *v39; // r0
  _DWORD *v40; // r0
  int v41; // r3
  _DWORD *v42; // r1
  _DWORD *v43; // r4
  _DWORD v44[4]; // [sp+Ch] [bp-4Ch] BYREF
  _DWORD v45[11]; // [sp+1Ch] [bp-3Ch] BYREF
  void *varg_r1; // [sp+4Ch] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+4Ch] [bp-Ch]
  int varg_r2; // [sp+50h] [bp-8h]
  int varg_r3; // [sp+54h] [bp-4h]
  va_list va1; // [sp+58h] [bp+0h] BYREF

  va_start(va1, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(va1, void *);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v2 = sub_132FC(v44, (void **)va1);
  v3 = sub_132FC(v45, (void **)varg_r1a);
  sub_44004(a1, *v3, v3[1], v3[2], *v2, v2[1], v2[2]);
  v4 = operator new(0x34u);
  if ( v4 != nullptr )
    v5 = sub_45EC0(v4);
  else
    v5 = nullptr;
  v6 = a1[32];
  a1[258] = v5;
  sub_46294(v5, 0, 0, v6);
  *(_DWORD *)(a1[258] + 24) = 0;
  *(_DWORD *)(a1[258] + 4) = 5;
  *(_DWORD *)(a1[258] + 8) = 30;
  *(_DWORD *)(a1[258] + 28) = 1;
  v7 = (_DWORD *)a1[258];
  if ( *v7 == 0 )
    __debugbreak();
  v7[9] = *(_DWORD *)(*v7 + 4);
  v8 = (_DWORD *)a1[258];
  if ( *v8 == 0 )
    __debugbreak();
  v8[10] = *(_DWORD *)(*v8 + 8) / v8[1];
  v9 = operator new(0x34u);
  if ( v9 != nullptr )
    v10 = sub_45EC0(v9);
  else
    v10 = nullptr;
  v11 = a1[15];
  a1[260] = v10;
  sub_46294(v10, 0, 0, v11);
  *(_DWORD *)(a1[260] + 24) = 0;
  *(_DWORD *)(a1[260] + 4) = 5;
  *(_DWORD *)(a1[260] + 8) = 13;
  *(_DWORD *)(a1[260] + 28) = 1;
  v12 = (_DWORD *)a1[260];
  if ( *v12 == 0 )
    __debugbreak();
  v12[9] = *(_DWORD *)(*v12 + 4);
  v13 = (_DWORD *)a1[260];
  if ( *v13 == 0 )
    __debugbreak();
  v13[10] = *(_DWORD *)(*v13 + 8) / v13[1];
  v14 = operator new(0x34u);
  if ( v14 != nullptr )
    v15 = sub_45EC0(v14);
  else
    v15 = nullptr;
  v16 = a1[33];
  a1[259] = v15;
  sub_46294(v15, 0, 0, v16);
  *(_DWORD *)(a1[259] + 24) = 0;
  *(_DWORD *)(a1[259] + 4) = 5;
  *(_DWORD *)(a1[259] + 8) = 31;
  *(_DWORD *)(a1[259] + 28) = 1;
  v17 = (_DWORD *)a1[259];
  if ( *v17 == 0 )
    __debugbreak();
  v17[9] = *(_DWORD *)(*v17 + 4);
  v18 = (_DWORD *)a1[259];
  if ( *v18 == 0 )
    __debugbreak();
  v18[10] = *(_DWORD *)(*v18 + 8) / v18[1];
  v19 = operator new(0x34u);
  if ( v19 != nullptr )
    v20 = sub_45EC0(v19);
  else
    v20 = nullptr;
  v21 = a1[38];
  a1[261] = v20;
  sub_46294(v20, 0, 0, v21);
  *(_DWORD *)(a1[261] + 24) = 0;
  *(_DWORD *)(a1[261] + 4) = 5;
  *(_DWORD *)(a1[261] + 8) = 36;
  *(_DWORD *)(a1[261] + 28) = 1;
  v22 = (_DWORD *)a1[261];
  if ( *v22 == 0 )
    __debugbreak();
  v22[9] = *(_DWORD *)(*v22 + 4);
  v23 = (_DWORD *)a1[261];
  if ( *v23 == 0 )
    __debugbreak();
  v23[10] = *(_DWORD *)(*v23 + 8) / v23[1];
  v24 = operator new(0x34u);
  if ( v24 != nullptr )
    v25 = sub_45EC0(v24);
  else
    v25 = nullptr;
  v26 = a1[16];
  a1[262] = v25;
  sub_46294(v25, 0, 0, v26);
  *(_DWORD *)(a1[262] + 24) = 0;
  *(_DWORD *)(a1[262] + 4) = 1;
  *(_DWORD *)(a1[262] + 8) = 14;
  *(_DWORD *)(a1[262] + 28) = 1;
  *(_DWORD *)(a1[262] + 12) = 16;
  *(_DWORD *)(a1[262] + 16) = 48;
  v27 = (_DWORD *)a1[262];
  if ( *v27 == 0 )
    __debugbreak();
  v27[9] = *(_DWORD *)(*v27 + 4);
  v28 = (_DWORD *)a1[262];
  if ( *v28 == 0 )
    __debugbreak();
  v28[10] = *(_DWORD *)(*v28 + 8) / v28[1];
  v29 = operator new(0x34u);
  if ( v29 != nullptr )
    v30 = sub_45EC0(v29);
  else
    v30 = nullptr;
  v31 = a1[28];
  a1[279] = v30;
  sub_46294(v30, 0, 0, v31);
  *(_DWORD *)(a1[279] + 24) = 0;
  *(_DWORD *)(a1[279] + 4) = 1;
  *(_DWORD *)(a1[279] + 8) = 26;
  *(_DWORD *)(a1[279] + 28) = 1;
  *(_DWORD *)(a1[279] + 12) = 16;
  *(_DWORD *)(a1[279] + 16) = 48;
  v32 = (_DWORD *)a1[279];
  if ( *v32 == 0 )
    __debugbreak();
  v32[9] = *(_DWORD *)(*v32 + 4);
  v33 = (_DWORD *)a1[279];
  if ( *v33 == 0 )
    __debugbreak();
  v33[10] = *(_DWORD *)(*v33 + 8) / v33[1];
  v34 = operator new(0x34u);
  if ( v34 != nullptr )
    v35 = sub_45EC0(v34);
  else
    v35 = nullptr;
  v36 = a1[29];
  a1[280] = v35;
  sub_46294(v35, 0, 0, v36);
  *(_DWORD *)(a1[280] + 24) = 0;
  *(_DWORD *)(a1[280] + 4) = 1;
  *(_DWORD *)(a1[280] + 8) = 27;
  *(_DWORD *)(a1[280] + 28) = 1;
  *(_DWORD *)(a1[280] + 12) = 16;
  *(_DWORD *)(a1[280] + 16) = 48;
  v37 = (_DWORD *)a1[280];
  if ( *v37 == 0 )
    __debugbreak();
  v37[9] = *(_DWORD *)(*v37 + 4);
  v38 = (_DWORD *)a1[280];
  if ( *v38 == 0 )
    __debugbreak();
  v38[10] = *(_DWORD *)(*v38 + 8) / v38[1];
  v39 = operator new(0x34u);
  if ( v39 != nullptr )
    v40 = sub_45EC0(v39);
  else
    v40 = nullptr;
  v41 = a1[120];
  a1[283] = v40;
  sub_46294(v40, 0, 0, v41);
  *(_DWORD *)(a1[283] + 24) = 0;
  *(_DWORD *)(a1[283] + 4) = 1;
  *(_DWORD *)(a1[283] + 8) = 118;
  *(_DWORD *)(a1[283] + 28) = 1;
  v42 = (_DWORD *)a1[283];
  if ( *v42 == 0 )
    __debugbreak();
  v42[9] = *(_DWORD *)(*v42 + 4);
  v43 = (_DWORD *)a1[283];
  if ( *v43 == 0 )
    __debugbreak();
  v43[10] = *(_DWORD *)(*v43 + 8) / v43[1];
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)va1);
}

