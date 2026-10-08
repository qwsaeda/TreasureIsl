/* ea:   0x0003EF44 */
/* name: sub_3EF44 */
/* size: 1240 */
int __fastcall sub_3EF44(int a1)
{
  int v2; // r1
  int v3; // r1
  int v4; // r0
  int v5; // r2
  _DWORD *v6; // r2
  int v7; // r1
  int (*v8)(); // r1
  int v9; // r4
  int result; // r0
  int v11; // r1
  int (*v12)(); // r9
  int v13; // r2
  int v14; // r0
  _DWORD *v15; // r2
  int v16; // t1
  int v17; // r1
  int (*v18)(); // r0
  int v19; // r4
  int v20; // r2
  int v21; // r0
  _DWORD *v22; // r2
  int v23; // t1
  int v24; // r1
  int (*v25)(); // r0
  int v26; // r4
  int v27; // r2
  int v28; // r0
  _DWORD *v29; // r2
  int v30; // t1
  int v31; // r1
  int (*v32)(); // r0
  int v33; // r4
  int v34; // r2
  int v35; // r0
  _DWORD *v36; // r2
  int v37; // t1
  int v38; // r1
  int (*v39)(); // r0
  int v40; // r4
  int v41; // r2
  int v42; // r0
  _DWORD *v43; // r2
  int v44; // t1
  int v45; // r1
  int (*v46)(); // r0
  int v47; // r4
  _DWORD *v48; // r5
  int v49; // r1
  int v50; // r0
  int v51; // r4
  int v52; // r3
  int v53; // r4
  int v54; // [sp+Ch] [bp-B4h] BYREF
  int v55; // [sp+10h] [bp-B0h]
  int v56; // [sp+14h] [bp-ACh]
  int v57; // [sp+18h] [bp-A8h]
  _DWORD v58[4]; // [sp+1Ch] [bp-A4h] BYREF
  _DWORD v59[4]; // [sp+2Ch] [bp-94h] BYREF
  _DWORD v60[4]; // [sp+3Ch] [bp-84h] BYREF
  _DWORD v61[4]; // [sp+4Ch] [bp-74h] BYREF
  _DWORD v62[2]; // [sp+5Ch] [bp-64h] BYREF
  int v63; // [sp+64h] [bp-5Ch]
  int v64; // [sp+68h] [bp-58h]
  _DWORD v65[4]; // [sp+6Ch] [bp-54h] BYREF
  _DWORD v66[4]; // [sp+7Ch] [bp-44h] BYREF
  _DWORD v67[13]; // [sp+8Ch] [bp-34h] BYREF

  v2 = *(_DWORD *)(a1 + 72);
  if ( v2 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(*(_DWORD *)(a1 + 72));
    v3 = *(_DWORD *)(a1 + 68);
    v62[1] = 25;
    v62[0] = 0;
    v63 = 40 * (v3 / 4);
    v64 = v63 + 40;
    v54 = 110;
    v4 = *(_DWORD *)(a1 + 52);
    v56 = 20;
    v55 = 135;
    v5 = *(_DWORD *)(a1 + 32);
    v57 = 60;
    v6 = (_DWORD *)(v5 + 4 * (v4 + 5));
    if ( *v6 == 0 )
      __debugbreak();
    v7 = *(_DWORD *)(*v6 + 12);
    if ( v7 == 16 )
    {
      v8 = sub_5BC14;
    }
    else if ( v7 == 8 )
    {
      v8 = sub_5D9C4;
    }
    else
    {
      v8 = nullptr;
    }
    v9 = *(_DWORD *)(a1 + 8);
    *(_DWORD *)(a1 + 12) = v8;
    (*(void (__fastcall **)(int, int *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v9 + 20))(
      v9,
      &v54,
      *v6,
      v62,
      v8,
      &dword_81230,
      1);
    result = *(_DWORD *)(a1 + 52);
    v11 = *(_DWORD *)(a1 + 68) + 1;
    *(_DWORD *)(a1 + 68) = v11;
    if ( result >= 5 )
    {
      if ( v11 == 32 )
        *(_DWORD *)(a1 + 68) = 0;
    }
    else if ( v11 == 16 )
    {
      *(_DWORD *)(a1 + 68) = 0;
    }
  }
  else
  {
    if ( *(_DWORD *)(a1 + 48) != 0 )
      (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 48) + 12))(*(_DWORD *)(a1 + 48));
    v12 = nullptr;
    v13 = *(_DWORD *)(a1 + 32);
    v58[0] = 0;
    v58[1] = 60;
    v58[2] = 0;
    v58[3] = 57;
    v16 = *(_DWORD *)(v13 + 4);
    v15 = (_DWORD *)(v13 + 4);
    v14 = v16;
    if ( v16 == 0 )
      __debugbreak();
    v17 = *(_DWORD *)(v14 + 12);
    if ( v17 == 16 )
    {
      v18 = sub_5BC14;
    }
    else if ( v17 == 8 )
    {
      v18 = sub_5D9C4;
    }
    else
    {
      v18 = nullptr;
    }
    v19 = *(_DWORD *)(a1 + 8);
    *(_DWORD *)(a1 + 12) = v18;
    v66[0] = 18;
    v66[1] = 78;
    v66[2] = 11;
    v66[3] = 68;
    (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v19 + 20))(
      v19,
      v66,
      *v15,
      v58,
      v18,
      &dword_81230,
      1);
    v20 = *(_DWORD *)(a1 + 32);
    v23 = *(_DWORD *)(v20 + 8);
    v22 = (_DWORD *)(v20 + 8);
    v21 = v23;
    if ( v23 == 0 )
      __debugbreak();
    v24 = *(_DWORD *)(v21 + 12);
    if ( v24 == 16 )
    {
      v25 = sub_5BC14;
    }
    else if ( v24 == 8 )
    {
      v25 = sub_5D9C4;
    }
    else
    {
      v25 = nullptr;
    }
    v26 = *(_DWORD *)(a1 + 8);
    *(_DWORD *)(a1 + 12) = v25;
    v59[0] = 101;
    v59[1] = 161;
    v59[2] = 11;
    v59[3] = 68;
    (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v26 + 20))(
      v26,
      v59,
      *v22,
      v58,
      v25,
      &dword_81230,
      1);
    v27 = *(_DWORD *)(a1 + 32);
    v30 = *(_DWORD *)(v27 + 12);
    v29 = (_DWORD *)(v27 + 12);
    v28 = v30;
    if ( v30 == 0 )
      __debugbreak();
    v31 = *(_DWORD *)(v28 + 12);
    if ( v31 == 16 )
    {
      v32 = sub_5BC14;
    }
    else if ( v31 == 8 )
    {
      v32 = sub_5D9C4;
    }
    else
    {
      v32 = nullptr;
    }
    v33 = *(_DWORD *)(a1 + 8);
    *(_DWORD *)(a1 + 12) = v32;
    v60[0] = 18;
    v60[1] = 78;
    v60[2] = 115;
    v60[3] = 172;
    (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v33 + 20))(
      v33,
      v60,
      *v29,
      v58,
      v32,
      &dword_81230,
      1);
    v34 = *(_DWORD *)(a1 + 32);
    v37 = *(_DWORD *)(v34 + 16);
    v36 = (_DWORD *)(v34 + 16);
    v35 = v37;
    if ( v37 == 0 )
      __debugbreak();
    v38 = *(_DWORD *)(v35 + 12);
    if ( v38 == 16 )
    {
      v39 = sub_5BC14;
    }
    else if ( v38 == 8 )
    {
      v39 = sub_5D9C4;
    }
    else
    {
      v39 = nullptr;
    }
    v40 = *(_DWORD *)(a1 + 8);
    *(_DWORD *)(a1 + 12) = v39;
    v61[0] = 101;
    v61[1] = 161;
    v61[3] = 172;
    v61[2] = 115;
    (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v40 + 20))(
      v40,
      v61,
      *v36,
      v58,
      v39,
      &dword_81230,
      1);
    if ( *(_DWORD *)(a1 + 64) == 5 )
    {
      v41 = *(_DWORD *)(a1 + 32);
      v44 = *(_DWORD *)(v41 + 20);
      v43 = (_DWORD *)(v41 + 20);
      v42 = v44;
      if ( v44 == 0 )
        __debugbreak();
      v45 = *(_DWORD *)(v42 + 12);
      if ( v45 == 16 )
      {
        v46 = sub_5BC14;
      }
      else if ( v45 == 8 )
      {
        v46 = sub_5D9C4;
      }
      else
      {
        v46 = nullptr;
      }
      *(_DWORD *)(a1 + 12) = v46;
      v56 = 75;
      v47 = *(_DWORD *)(a1 + 8);
      v54 = 58;
      v55 = 118;
      v57 = 132;
      (*(void (__fastcall **)(int, int *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v47 + 20))(
        v47,
        &v54,
        *v43,
        v58,
        v46,
        &dword_81230,
        1);
    }
    v48 = *(_DWORD **)(a1 + 32);
    v65[0] = 0;
    v65[1] = 20;
    v65[2] = 0;
    v65[3] = 33;
    if ( *v48 == 0 )
      __debugbreak();
    v49 = *(_DWORD *)(*v48 + 12);
    if ( v49 == 16 )
    {
      v12 = sub_5BC14;
    }
    else if ( v49 == 8 )
    {
      v12 = sub_5D9C4;
    }
    v50 = *(_DWORD *)(a1 + 56);
    v51 = *(_DWORD *)(a1 + 60);
    *(_DWORD *)(a1 + 12) = v12;
    v67[2] = v51;
    v52 = v51 + 33;
    v53 = *(_DWORD *)(a1 + 8);
    v67[0] = v50 - 10;
    v67[3] = v52;
    v67[1] = v50 + 10;
    return (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v53 + 20))(
             v53,
             v67,
             *v48,
             v65,
             v12,
             &dword_81230,
             1);
  }
  return result;
}

