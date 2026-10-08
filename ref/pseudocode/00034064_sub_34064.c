/* ea:   0x00034064 */
/* name: sub_34064 */
/* size: 732 */
void __fastcall sub_34064(_DWORD *a1, int a2, int a3)
{
  int *v4; // r4
  int v5; // r0
  int v6; // r1
  int v7; // r9
  int v8; // r6
  int v9; // r5
  int v10; // r4
  int v11; // r3
  int v12; // r2
  int v13; // r4
  int v14; // r3
  int v15; // r2
  int v16; // r7
  int v17; // r8
  int *i; // r6
  int v19; // r5
  int v20; // r0
  _BYTE v21[4]; // [sp+Ch] [bp-8Ch] BYREF
  int v22; // [sp+10h] [bp-88h]
  int v23; // [sp+14h] [bp-84h]
  int *v24; // [sp+1Ch] [bp-7Ch] BYREF
  int *v25; // [sp+20h] [bp-78h]
  int v26; // [sp+2Ch] [bp-6Ch]
  int v27; // [sp+30h] [bp-68h]
  int v28; // [sp+34h] [bp-64h]
  int v29; // [sp+38h] [bp-60h]
  _DWORD v30[4]; // [sp+3Ch] [bp-5Ch] BYREF
  _DWORD v31[4]; // [sp+4Ch] [bp-4Ch] BYREF
  _DWORD v32[15]; // [sp+5Ch] [bp-3Ch] BYREF

  v32[14] = a3;
  v23 = a2;
  v4 = a1 + 4;
  v5 = a1[5];
  v6 = *v4;
  v22 = 0;
  if ( v6 != v5 )
  {
    sub_34344(&v24, v21);
    if ( a1[7] == a1[8] )
    {
      sub_36384(&v24, v4);
      v7 = a1[12];
    }
    else
    {
      sub_36384(&v24, a1 + 7);
      v7 = a1[13];
    }
    v26 = 0;
    v28 = 0;
    v8 = v25 - v24;
    v32[0] = 0;
    v32[2] = 0;
    v27 = 54;
    v29 = 16;
    v32[1] = 40;
    v32[3] = 11;
    if ( v8 > 0 )
    {
      v9 = v22;
      do
      {
        v10 = a1[14];
        v11 = v9 + a1[15] + 16;
        v30[2] = v9 + a1[15];
        v12 = a1[10];
        v30[0] = v10;
        v30[1] = v10 + 54;
        v30[3] = v11;
        if ( v12 == 0 )
          __debugbreak();
        (*(void (__fastcall **)(int, _DWORD *))(*(_DWORD *)v23 + 20))(v23, v30);
        --v8;
        v9 += 16;
      }
      while ( v8 != 0 );
    }
    v13 = a1[14];
    v14 = a1[15] + 16 * v7 + 16;
    v31[2] = a1[15] + 16 * v7;
    v15 = a1[11];
    v31[0] = v13;
    v31[1] = v13 + 54;
    v31[3] = v14;
    if ( v15 == 0 )
      __debugbreak();
    v16 = v22;
    v17 = v23;
    (*(void (__fastcall **)(int, _DWORD *))(*(_DWORD *)v23 + 20))(v23, v31);
    for ( i = v24; i != v25; ++i )
    {
      v19 = *(_DWORD *)sub_33C30(a1 + 1, i);
      sub_2F4A4(v30, a1[16] + a1[14], a1[17] + a1[15] + v16, a1[16] + a1[14] + 40, a1[17] + a1[15] + v16 + 11);
      if ( v19 == 0 )
        __debugbreak();
      v20 = sub_290B8(*(_DWORD *)(v19 + 12), 2, 0);
      (*(void (__fastcall **)(int, _DWORD *, int, _DWORD *, int, int *, int))(*(_DWORD *)v17 + 20))(
        v17,
        v30,
        v19,
        v32,
        v20,
        &dword_81230,
        1);
      v16 += 16;
    }
    sub_34340((void **)&v24);
  }
}

