/* ea:   0x00050868 */
/* name: sub_50868 */
/* size: 1288 */
int __fastcall sub_50868(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  _DWORD *v5; // lr
  int v6; // r10
  int v7; // r3
  int v8; // r2
  int v9; // r0
  int v10; // r4
  int v11; // r0
  int v12; // r1
  int v13; // r1
  int v14; // r0
  int v15; // r1
  int v16; // r2
  int v17; // r0
  int v18; // r8
  int v19; // r0
  int v20; // r7
  int v21; // r6
  int v22; // r5
  int v23; // r4
  int v24; // r2
  int v25; // r1
  int (*v26)(); // r0
  int v27; // r2
  _DWORD *v28; // r4
  int v29; // r7
  int v30; // r6
  int v31; // r5
  int v32; // r8
  int v33; // r3
  int v34; // r0
  int v35; // r2
  int v36; // r9
  int v37; // r3
  _DWORD *v38; // r10
  int v39; // r7
  int v40; // r6
  int v41; // r4
  int v42; // r5
  int v43; // r2
  int v44; // r0
  int v45; // r3
  int v46; // r0
  int v47; // r2
  int v52; // [sp+1Ch] [bp-74h]
  int v53; // [sp+20h] [bp-70h]
  int v54; // [sp+2Ch] [bp-64h] BYREF
  int v55; // [sp+30h] [bp-60h]
  int v56; // [sp+34h] [bp-5Ch]
  int v57; // [sp+38h] [bp-58h]
  int v58; // [sp+3Ch] [bp-54h] BYREF
  int v59; // [sp+40h] [bp-50h]
  int v60; // [sp+44h] [bp-4Ch]
  int v61; // [sp+48h] [bp-48h]
  int v62; // [sp+4Ch] [bp-44h]
  int v63; // [sp+50h] [bp-40h]
  int v64; // [sp+54h] [bp-3Ch]
  int v65; // [sp+58h] [bp-38h]
  _DWORD v66[13]; // [sp+5Ch] [bp-34h] BYREF

  v5 = a1;
  v6 = -1;
  v52 = a1[31] * a1[18];
  switch ( a1[44] )
  {
    case 2:
      v6 = 5;
      break;
    case 4:
      v6 = 3;
      break;
    case 5:
      v6 = 0;
      break;
    case 6:
      v6 = 4;
      break;
    case 7:
      v6 = 2;
      break;
    case 8:
      v6 = 1;
      break;
    default:
      break;
  }
  v7 = a1[29];
  if ( v7 % 4 == 0 )
  {
    v8 = a1[27];
    v9 = v8 + 1;
    if ( v8 >= v5[24] - 1 )
      v9 = 0;
    v5[27] = v9;
  }
  v10 = v5[19];
  v53 = v10 * v5[27];
  if ( v6 >= 0 && v7 % 8 == 0 )
  {
    v11 = v5[28];
    v12 = v11 + 1;
    if ( v11 >= 2 )
      v12 = 0;
    v5[28] = v12;
  }
  v13 = v5[10];
  if ( v7 < 23 )
    v5[29] = v7 + 1;
  v14 = v5[11];
  if ( v7 >= 23 )
    v5[29] = 0;
  if ( v13 == v14 )
  {
    v15 = v5[13];
    if ( v15 == 0 )
      __debugbreak();
    v16 = *(_DWORD *)(v5[13] + 8);
    if ( v15 == 0 )
      __debugbreak();
    v17 = *(_DWORD *)(v15 + 4);
    v54 = 0;
    v55 = v17;
    v56 = 0;
    v57 = v16;
    v18 = sub_53570(v5, v10);
    v19 = a1[13];
    if ( v19 == 0 )
      __debugbreak();
    v20 = *(_DWORD *)(v19 + 8);
    v21 = (*(int (__fastcall **)(_DWORD *))(*a1 + 20))(a1);
    v22 = a1[13];
    if ( v22 == 0 )
      __debugbreak();
    v23 = sub_53570(a1, a1[19]);
    v58 = (*(int (__fastcall **)(_DWORD *))(*a1 + 20))(a1) + a4;
    v24 = 3 * a1[19];
    v59 = v21 + a4 + *(_DWORD *)(v22 + 4);
    v60 = v23 + v24 / 4 + a5;
    v61 = v20 + v24 / 4 + v18 + a5;
    if ( a1[13] == 0 )
      __debugbreak();
    v25 = *(_DWORD *)(a1[13] + 12);
    if ( v25 == 16 )
    {
      v26 = sub_5BC14;
    }
    else if ( v25 == 8 )
    {
      v26 = sub_5D9C4;
    }
    else
    {
      v26 = nullptr;
    }
    (*(void (__fastcall **)(int, int *, _DWORD, int *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
      a2,
      &v58,
      a1[13],
      &v54,
      v26,
      &dword_81230,
      1);
    v5 = a1;
  }
  if ( v6 < 0 )
  {
    v32 = a4;
    v36 = a2;
  }
  else
  {
    v27 = 14 * v5[28];
    v58 = 16 * v6;
    v59 = 16 * v6 + 16;
    v60 = v27;
    v61 = v27 + 14;
    v28 = a1;
    v29 = sub_53570(v5, v5[19]);
    v30 = (*(int (__fastcall **)(_DWORD *))(*a1 + 20))(a1);
    v31 = sub_53570(v28, v28[19]);
    v32 = a4;
    v33 = (*(int (__fastcall **)(_DWORD *))(*v28 + 20))(v28);
    v34 = a1[18];
    v54 = v33 + v34 / 2 + a4;
    v57 = v29 + a5;
    v35 = a1[14];
    v55 = v30 + v34 / 2 + a4 + 16;
    v56 = v31 + a5 - 14;
    if ( v35 == 0 )
      __debugbreak();
    v36 = a2;
    (*(void (__fastcall **)(int, int *))(*(_DWORD *)a2 + 20))(a2, &v54);
    v5 = a1;
  }
  v37 = v5[19];
  v63 = v5[18] + v52;
  v64 = v53;
  v62 = v52;
  v65 = v37 + v53;
  v38 = a1;
  v39 = sub_53570(v5, v37);
  v40 = (*(int (__fastcall **)(_DWORD *))(*a1 + 20))(a1);
  v41 = sub_53570(v38, v38[19]);
  v42 = (*(int (__fastcall **)(_DWORD *))(*v38 + 20))(v38) + v32;
  v43 = a1[19];
  v44 = v40 + a1[18];
  v66[0] = v42;
  v45 = v44 + v32;
  v66[2] = v41 + a5;
  v46 = v39 + v43;
  v47 = a1[10];
  v66[1] = v45;
  v66[3] = v46 + a5;
  if ( v47 == 0 )
    __debugbreak();
  return (*(int (__fastcall **)(int, _DWORD *))(*(_DWORD *)v36 + 20))(v36, v66);
}

