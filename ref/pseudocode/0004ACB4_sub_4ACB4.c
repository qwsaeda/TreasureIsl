/* ea:   0x0004ACB4 */
/* name: sub_4ACB4 */
/* size: 444 */
void __fastcall sub_4ACB4(_DWORD **a1)
{
  _DWORD **v2; // r1
  _DWORD *v3; // r0
  int v4; // r10
  _DWORD *v5; // r4
  int i; // r5
  _DWORD *v7; // r1
  int v8; // r7
  int v9; // r6
  void **v10; // r5
  void **v11; // r4
  _DWORD *v12; // r1
  int v13; // r5
  _DWORD *v14; // r4
  void **v15; // r4
  _DWORD *v16; // [sp+0h] [bp-134h] BYREF
  char v17; // [sp+4h] [bp-130h] BYREF
  void *v18; // [sp+8h] [bp-12Ch] BYREF
  int v19; // [sp+Ch] [bp-128h] BYREF
  _DWORD v20[64]; // [sp+10h] [bp-124h] BYREF
  char v21; // [sp+110h] [bp-24h] BYREF

  v2 = (_DWORD **)*a1;
  v3 = (_DWORD *)**a1;
  if ( v3 != v2 && (_DWORD **)*v3 != v2 )
  {
    sub_4A3A8(&v16);
    v4 = 64;
    v5 = v20;
    for ( i = 64; i != 0; --i )
      sub_4AE70(v5++, &v17);
    v7 = v16;
    v8 = 0;
    while ( (_DWORD *)**a1 != *a1 )
    {
      sub_4AF80(&v16, *v7, a1);
      v9 = 0;
      if ( v8 > 0 )
      {
        v10 = (void **)v20;
        do
        {
          if ( *(void **)*v10 == *v10 )
            break;
          sub_4AFCC(v10);
          v11 = v10;
          ++v9;
          ++v10;
          sub_4B09C(&v18, &v16);
          v16 = *v11;
          *v11 = v18;
        }
        while ( v9 < v8 );
      }
      sub_4B09C(&v19, &v16);
      v7 = (_DWORD *)v20[v9];
      v20[v9] = v19;
      v16 = v7;
      if ( v9 == v8 )
        ++v8;
    }
    if ( v8 > 1 )
    {
      v12 = v20;
      v13 = v8 - 1;
      do
      {
        v14 = v12 + 1;
        sub_4AFCC(v12 + 1);
        --v13;
        v12 = v14;
      }
      while ( v13 != 0 );
    }
    sub_4B080(a1, &v20[v8 - 1]);
    v15 = (void **)&v21;
    do
    {
      sub_4A3D8(--v15);
      --v4;
    }
    while ( v4 != 0 );
    sub_4A3D8((void **)&v16);
  }
}

