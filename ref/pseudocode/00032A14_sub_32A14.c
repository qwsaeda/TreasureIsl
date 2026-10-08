/* ea:   0x00032A14 */
/* name: sub_32A14 */
/* size: 952 */
int __fastcall sub_32A14(_DWORD *a1, int a2)
{
  int v4; // r1
  int v5; // r2
  int v6; // r3
  int v7; // r4
  int v8; // r1
  int (*v9)(); // r0
  int v10; // r2
  int v11; // r4
  int v12; // r3
  int v13; // r1
  int v14; // r2
  int v15; // r3
  int v16; // r5
  int v17; // r0
  int v18; // r0
  int v19; // r1
  int v20; // r2
  int v21; // r6
  int v22; // r0
  int v23; // r1
  int v24; // r5
  int v25; // r2
  int v26; // r3
  int v27; // r2
  int v28; // r1
  int v29; // r6
  int v30; // r5
  int v31; // r2
  int v32; // r3
  int v33; // r0
  int v34; // r1
  int v35; // r5
  int v36; // r2
  int v39; // [sp+14h] [bp-A4h] BYREF
  int v40; // [sp+18h] [bp-A0h]
  int v41; // [sp+1Ch] [bp-9Ch]
  int v42; // [sp+20h] [bp-98h]
  int v43; // [sp+24h] [bp-94h] BYREF
  int v44; // [sp+28h] [bp-90h]
  int v45; // [sp+2Ch] [bp-8Ch]
  int v46; // [sp+30h] [bp-88h]
  int v47; // [sp+34h] [bp-84h]
  int v48; // [sp+38h] [bp-80h]
  int v49; // [sp+3Ch] [bp-7Ch]
  int v50; // [sp+40h] [bp-78h]
  _DWORD v51[4]; // [sp+44h] [bp-74h] BYREF
  int v52; // [sp+54h] [bp-64h] BYREF
  int v53; // [sp+58h] [bp-60h]
  int v54; // [sp+5Ch] [bp-5Ch]
  int v55; // [sp+60h] [bp-58h]
  int v56; // [sp+64h] [bp-54h]
  int v57; // [sp+68h] [bp-50h]
  int v58; // [sp+6Ch] [bp-4Ch]
  int v59; // [sp+70h] [bp-48h]
  _DWORD v60[4]; // [sp+74h] [bp-44h] BYREF
  _DWORD v61[13]; // [sp+84h] [bp-34h] BYREF

  v4 = a1[6];
  v5 = a1[11];
  v6 = a1[12];
  v53 = a1[5];
  v7 = a1[13];
  v55 = v4;
  v60[1] = v5 + v53;
  v52 = 0;
  v54 = 0;
  v60[0] = v5;
  v60[2] = v6;
  v60[3] = v6 + v4;
  if ( v7 == 0 )
    __debugbreak();
  v8 = *(_DWORD *)(v7 + 12);
  if ( v8 == 16 )
  {
    v9 = sub_5BC14;
  }
  else if ( v8 == 8 )
  {
    v9 = sub_5D9C4;
  }
  else
  {
    v9 = nullptr;
  }
  v10 = v7;
  v11 = *(_DWORD *)(*(_DWORD *)a2 + 20);
  ((void (__fastcall *)(int, _DWORD *, int, int *, int (*)(), int *, int))v11)(a2, v60, v10, &v52, v9, &dword_81230, 1);
  v12 = a1[2];
  if ( v12 <= 0 )
  {
    v22 = a1[11];
    v23 = a1[12];
    v24 = a1[4];
    v40 = a1[3];
    v44 = v40 + v22 - 9;
    v25 = a1[14];
    v39 = 0;
    v41 = 0;
    v42 = v24;
    v43 = v22 - 9;
    v45 = v23 + 13;
    v46 = v24 + v23 + 13;
    if ( v25 == 0 )
      __debugbreak();
    (*(void (__fastcall **)(int, int *))(*(_DWORD *)a2 + 20))(a2, &v43);
    v26 = a1[9];
    v27 = a1[11];
    v28 = v26 * *a1;
    v29 = a1[10];
    v30 = a1[12];
    v49 = 0;
    v50 = v29;
    v47 = v28;
    v48 = v26 + v28;
    v51[0] = v27 + 17;
    v51[1] = v26 + v27 + 17;
    v31 = a1[17];
    v51[3] = v29 + v30 + 37;
    v51[2] = v30 + 37;
    if ( v31 == 0 )
      __debugbreak();
    v21 = a2;
    (*(void (__fastcall **)(int, _DWORD *))(*(_DWORD *)a2 + 20))(a2, v51);
    v32 = a1[2];
    if ( v32 + 48 > 0 )
      a1[2] = v32 - 1;
  }
  else
  {
    v13 = a1[1];
    v14 = v12 - 1;
    v15 = a1[3];
    v16 = a1[4];
    if ( v13 >= 2 )
      a1[1] = 0;
    else
      v11 = v13 * a1[3];
    v17 = a1[1];
    if ( v13 >= 2 )
      v11 = 0;
    a1[2] = v14;
    v43 = v11;
    a1[1] = v17 + 1;
    v44 = v15 + v11;
    v18 = a1[11];
    v19 = a1[12];
    v45 = 0;
    v46 = v16;
    v39 = v18 - 9;
    v40 = v15 + v18 - 9;
    v20 = a1[15];
    v41 = v19 + 13;
    v42 = v16 + v19 + 13;
    if ( v20 == 0 )
      __debugbreak();
    (*(void (__fastcall **)(int, int *))(*(_DWORD *)a2 + 20))(a2, &v39);
    v21 = a2;
  }
  v33 = a1[11];
  v34 = a1[12];
  v35 = a1[8];
  v57 = a1[7];
  v61[1] = v57 + v33 + 40;
  v36 = a1[16];
  v56 = 0;
  v58 = 0;
  v59 = v35;
  v61[0] = v33 + 40;
  v61[2] = v34 - 2;
  v61[3] = v35 + v34 - 2;
  if ( v36 == 0 )
    __debugbreak();
  return (*(int (__fastcall **)(int, _DWORD *))(*(_DWORD *)v21 + 20))(v21, v61);
}

