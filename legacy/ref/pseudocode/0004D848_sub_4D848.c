/* ea:   0x0004D848 */
/* name: sub_4D848 */
/* size: 1168 */
void __fastcall sub_4D848(_DWORD *a1, int a2, _WORD *a3)
{
  unsigned __int8 **v6; // r6
  char v7; // r8
  int v8; // r4
  unsigned int v9; // r1
  int v10; // r0
  int v11; // r5
  unsigned __int8 *v12; // r4
  _DWORD *v13; // r3
  char *v14; // r5
  int v15; // r0
  signed int v16; // r4
  int i; // [sp+4h] [bp-54h] BYREF
  int v18; // [sp+8h] [bp-50h] BYREF
  int v19; // [sp+Ch] [bp-4Ch] BYREF
  _DWORD v20[2]; // [sp+10h] [bp-48h] BYREF
  _DWORD *v21; // [sp+18h] [bp-40h] BYREF
  _BYTE *v22; // [sp+1Ch] [bp-3Ch]
  void *v23[12]; // [sp+28h] [bp-30h] BYREF

  *(_DWORD *)a2 = 0;
  *a3 = 0;
  v6 = (unsigned __int8 **)(a1 + 41);
  sub_4A34C(v23);
  v7 = 1;
  v18 = 1;
  sub_406DC((int *)v23, &v18);
  for ( i = 2; i <= 4; ++i )
  {
    v8 = *sub_1DE18((_DWORD *)(a1[6] + 496), &v19);
    if ( *sub_1E48C(a1[6] + 496, v20, (int)&i) != v8 )
      sub_406DC((int *)v23, &i);
  }
  v9 = rand() % (unsigned int)(((char *)v23[1] - (char *)v23[0]) >> 2);
  v10 = a1[2];
  v11 = 1;
  v12 = *v6;
  i = *((_DWORD *)v23[0] + v9);
  if ( i == v10 )
    v11 = 2;
  while ( v12 != (unsigned __int8 *)a1[42] )
  {
    v13 = (_DWORD *)sub_173C4(a1[6] + 652, *v12);
    *(_DWORD *)a2 = v13;
    if ( v13[13] != 7 && v13[8] != 4 && v13[14] == v11 )
      break;
    *(_DWORD *)a2 = 0;
    ++v12;
  }
  if ( *(_DWORD *)a2 == 0 )
    goto LABEL_43;
  sub_4A4E8(&v21);
  v14 = (char *)*sub_1DDF0(a1[6] + 508, v20);
  v15 = *(_DWORD *)(*(_DWORD *)a2 + 32);
  if ( v15 == 2 )
  {
    while ( v14 != (char *)*sub_1DE08((_DWORD *)(a1[6] + 508), v20) )
    {
      if ( *(unsigned __int8 *)(*((_DWORD *)v14 + 5) + 50) == i )
        sub_4A2B0((int)&v21, (_DWORD *)v14 + 5);
      v14 = (char *)sub_16A30(v14);
    }
    if ( v21 != (_DWORD *)v22 )
      v7 = 0;
    if ( v7 == 0 )
    {
      *a3 = *(_WORD *)(v21[rand() % (unsigned int)((v22 - (_BYTE *)v21) >> 2)] + 20);
      sub_4A4FC((void **)&v21);
      sub_30BC4(v23);
      return;
    }
    goto LABEL_42;
  }
  if ( v15 != 3 )
  {
    if ( v15 != 4 )
    {
      *a3 = i;
      sub_4A4FC((void **)&v21);
      sub_30BC4(v23);
      return;
    }
    while ( v14 != *(char **)(a1[6] + 508) )
    {
      sub_4A2B0((int)&v21, (_DWORD *)v14 + 5);
      v14 = (char *)sub_16A30(v14);
    }
    if ( v21 != (_DWORD *)v22 )
      v7 = 0;
    if ( v7 == 0 )
    {
      *a3 = *(_WORD *)(*(_DWORD *)(v21[rand() % (unsigned int)((v22 - (_BYTE *)v21) >> 2)] + 4) + 20);
      sub_4A4FC((void **)&v21);
      sub_30BC4(v23);
      return;
    }
LABEL_42:
    *(_DWORD *)a2 = 0;
    sub_4A4FC((void **)&v21);
LABEL_43:
    sub_30BC4(v23);
    return;
  }
  while ( v14 != *(char **)(a1[6] + 508) )
  {
    if ( *(unsigned __int8 *)(*((_DWORD *)v14 + 5) + 50) == i )
      sub_4A2B0((int)&v21, (_DWORD *)v14 + 5);
    v14 = (char *)sub_16A30(v14);
  }
  if ( v21 != (_DWORD *)v22 )
    v7 = 0;
  if ( v7 != 0 )
  {
    v16 = (unsigned int)rand() % *(_DWORD *)(a1[6] + 512) + 1;
    if ( v16 > 0 )
    {
      do
      {
        --v16;
        v14 = (char *)sub_17924(v14);
      }
      while ( v16 != 0 );
    }
    *a3 = *(unsigned __int8 *)(*((_DWORD *)v14 + 5) + 48);
    sub_4A4FC((void **)&v21);
    sub_30BC4(v23);
  }
  else
  {
    *a3 = *(unsigned __int8 *)(v21[rand() % (unsigned int)((v22 - (_BYTE *)v21) >> 2)] + 48);
    sub_4A4FC((void **)&v21);
    sub_30BC4(v23);
  }
}

