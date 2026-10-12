/* ea:   0x0001AA74 */
/* name: sub_1AA74 */
/* size: 320 */
// positive sp value has been detected, the output may be wrong!
void __fastcall sub_1AA74(
        _DWORD *a1,
        int *a2,
        int a3,
        int a4,
        int a5,
        int a6,
        int a7,
        int a8,
        int a9,
        int a10,
        int a11,
        int a12)
{
  int v14; // r5
  int v15; // r7
  int v16; // r3
  __int16 v17; // t1
  _WORD *v18; // r4
  int v19; // r3
  __int16 v20; // r2
  int v21; // r4
  int v22; // r3
  __int16 v23; // t1
  _WORD *v24; // r4
  int v25; // r3
  __int16 v26; // r2
  int v27; // r0
  _DWORD v28[3]; // [sp-4h] [bp-48h] BYREF
  _DWORD v29[5]; // [sp+8h] [bp-3Ch] BYREF
  int v30; // [sp+1Ch] [bp-28h]

  v14 = *a2;
  v15 = a2[1];
  for ( a1[262] = 0; v14 != v15; ++a1[262] )
  {
    LOBYTE(v30) = *(_BYTE *)(v14 + 32);
    v28[2] = *(_DWORD *)(v14 + 8);
    memcpy(v29, (const void *)(v14 + 12), sizeof(v29));
    v16 = *(_DWORD *)v14 - 2;
    do
    {
      v17 = *(_WORD *)(v16 + 2);
      v16 += 2;
    }
    while ( v17 != 0 );
    v18 = operator new(2 * (((unsigned int)(v16 - *(_DWORD *)v14) >> 1) + 1));
    v28[0] = v18;
    v19 = *(_DWORD *)v14 - (_DWORD)v18;
    do
    {
      v20 = *(_WORD *)((char *)v18 + v19);
      *v18++ = v20;
    }
    while ( v20 != 0 );
    v21 = *(_DWORD *)(v14 + 4);
    v22 = v21 - 2;
    do
    {
      v23 = *(_WORD *)(v22 + 2);
      v22 += 2;
    }
    while ( v23 != 0 );
    v24 = operator new(2 * (((unsigned int)(v22 - v21) >> 1) + 1));
    v28[1] = v24;
    v25 = *(_DWORD *)(v14 + 4) - (_DWORD)v24;
    do
    {
      v26 = *(_WORD *)((char *)v24 + v25);
      *v24++ = v26;
    }
    while ( v26 != 0 );
    ((void (__fastcall *)(_DWORD *, _DWORD *))sub_1AC70)(a1 + 258, v28);
    v14 += 36;
  }
  v27 = a1[262];
  a1[285] = a3;
  a1[257] = v27 != 0;
  __asm { POP             {R4-R10,PC} }
}

