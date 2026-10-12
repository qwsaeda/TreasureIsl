/* ea:   0x00068494 */
/* name: sub_68494 */
/* size: 388 */
int __fastcall sub_68494(unsigned int a1, unsigned __int8 *a2, unsigned int a3)
{
  unsigned __int8 *v3; // r6
  unsigned int v5; // r2
  unsigned int v6; // r3
  int v8; // r5
  unsigned int v9; // r4
  int v10; // r1
  int v11; // r2
  int v12; // r3
  int v13; // r1
  int v14; // r2
  int v15; // r3
  int v16; // r1
  int v17; // r2
  int v18; // r3
  int v19; // r1
  int v20; // r2
  int v21; // r3
  int v22; // r1
  int v23; // r2
  int v24; // r3
  int v25; // r1
  int v26; // r2
  int v27; // r3
  int v28; // r1
  int v29; // r2
  int v30; // r3
  int v31; // r1
  int v32; // r2
  int v33; // r3
  int v34; // r1
  int v35; // r2
  int v36; // r3
  int v37; // r1
  int v38; // r2
  int v39; // r3
  int v40; // t1

  v3 = a2;
  v5 = (unsigned __int16)a1;
  v6 = HIWORD(a1);
  if ( a2 == nullptr )
    return 1;
  for ( ; a3 != 0; v6 %= 0xFFF1u )
  {
    v8 = a3;
    if ( a3 >= 0x15B0 )
      v8 = 5552;
    a3 -= v8;
    if ( v8 >= 16 )
    {
      v9 = (unsigned int)v8 >> 4;
      v8 -= 16 * ((unsigned int)v8 >> 4);
      do
      {
        --v9;
        v10 = v5 + *v3;
        v11 = v6 + v10;
        v12 = v10 + v3[1];
        v13 = v11 + v12;
        v14 = v12 + v3[2];
        v15 = v13 + v14;
        v16 = v14 + v3[3];
        v17 = v15 + v16;
        v18 = v16 + v3[4];
        v19 = v17 + v18;
        v20 = v18 + v3[5];
        v21 = v19 + v20;
        v22 = v20 + v3[6];
        v23 = v21 + v22;
        v24 = v22 + v3[7];
        v25 = v23 + v24;
        v26 = v24 + v3[8];
        v27 = v25 + v26;
        v28 = v26 + v3[9];
        v29 = v27 + v28;
        v30 = v28 + v3[10];
        v31 = v29 + v30;
        v32 = v30 + v3[11];
        v33 = v31 + v32;
        v34 = v32 + v3[12];
        v35 = v33 + v34;
        v36 = v34 + v3[13];
        v37 = v35 + v36;
        v38 = v36 + v3[14];
        v39 = v37 + v38;
        v5 = v38 + v3[15];
        v6 = v39 + v5;
        v3 += 16;
      }
      while ( v9 != 0 );
    }
    for ( ; v8 != 0; v6 += v5 )
    {
      v40 = *v3++;
      --v8;
      v5 += v40;
    }
    v5 %= 0xFFF1u;
  }
  return v5 | (v6 << 16);
}

