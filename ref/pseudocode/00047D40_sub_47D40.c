/* ea:   0x00047D40 */
/* name: sub_47D40 */
/* size: 2324 */
void sub_47D40(int a1, _DWORD *a2, ...)
{
  int v4; // r0
  int v5; // r1
  int v6; // r9
  const void *v7; // r1
  unsigned int v8; // r8
  int v9; // r0
  _DWORD *i; // r2
  int v11; // r0
  int v12; // r0
  _DWORD *j; // r2
  int v14; // r0
  int v15; // r6
  int v16; // r7
  _DWORD *k; // r5
  int v18; // r4
  int v19; // r0
  _DWORD *m; // r2
  int v21; // r0
  int v22; // r3
  _DWORD *n; // r2
  int v24; // r0
  unsigned __int16 v25; // r5
  unsigned __int16 v26; // r6
  _DWORD *ii; // r4
  int v28; // r3
  unsigned int v29; // r2
  unsigned int v30; // r1
  unsigned int *v31; // r4
  unsigned __int16 v32; // r6
  int v33; // r5
  _DWORD *jj; // r4
  int v35; // r3
  __int16 v36; // r1
  int v37; // r0
  unsigned __int16 v38; // [sp+4h] [bp-5Ch] BYREF
  unsigned __int16 v39; // [sp+6h] [bp-5Ah] BYREF
  int v40; // [sp+8h] [bp-58h] BYREF
  _BYTE v41[16]; // [sp+Ch] [bp-54h] BYREF
  void *v42[13]; // [sp+1Ch] [bp-44h] BYREF
  void *varg_r2; // [sp+58h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+58h] [bp-8h]
  va_list varg_r3; // [sp+5Ch] [bp-4h] BYREF

  va_start(varg_r3, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(varg_r3, void *);
  if ( *(_DWORD *)(a1 + 20) == 0
    || sub_16C9C((unsigned __int16 *)(a1 + 136), 2) == 0
    || sub_16C9C((unsigned __int16 *)(a1 + 136), 0) == 0 )
  {
    goto LABEL_91;
  }
  v4 = rand();
  v5 = a2[7];
  v6 = v4 % 100 + 1;
  if ( v5 == 0 )
  {
    v7 = a2 + 4;
    goto LABEL_8;
  }
  if ( v5 == 1 )
  {
    v7 = (char *)a2 + 22;
LABEL_8:
    memcpy(&v41[4], v7, 6u);
  }
  v8 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  v39 = 0;
  v38 = 0;
  sub_32134((int)v42);
  if ( sub_31150(*(_DWORD *)(a1 + 20), 0) > 1 && v6 <= 10 * (8 - sub_31150(*(_DWORD *)(a1 + 20), 1)) )
  {
    if ( v8 >= 0x64 )
    {
      v9 = *(_DWORD *)(a1 + 20);
      for ( i = *(_DWORD **)(v9 + 28); i != *(_DWORD **)(v9 + 32); ++i )
      {
        if ( *i == 0 )
        {
          *i = 1;
          break;
        }
      }
      sub_16570((unsigned int *)(a1 + 144), 0x64u);
      sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
      v11 = sub_51C14(a1, v42);
      sub_1654C((unsigned int *)(a1 + 144), v11);
    }
LABEL_55:
    sub_16ED8(v42);
    sub_32158(v42);
    sub_327B0((void **)varg_r2a);
    return;
  }
  if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 2) != 0 && v6 <= 70 )
  {
    if ( v8 / 0xA * v41[4] >= 0x1F4 )
    {
      v12 = *(_DWORD *)(a1 + 20);
      for ( j = *(_DWORD **)(v12 + 28); j != *(_DWORD **)(v12 + 32); ++j )
      {
        if ( *j == 0 )
        {
          *j = 2;
          break;
        }
      }
      sub_16570((unsigned int *)(a1 + 144), 0x1F4u);
      sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
      v14 = sub_51C14(a1, v42);
      sub_1654C((unsigned int *)(a1 + 144), v14);
    }
    goto LABEL_55;
  }
  if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 3) != 0 )
  {
    v15 = 0;
    v16 = 0;
    for ( k = (_DWORD *)*sub_1DDF0((int)varg_r2a, &v40);
          k != (_DWORD *)*sub_1DE08((void **)varg_r2a, &v40);
          k = sub_16A30(k) )
    {
      v18 = k[5];
      if ( *(unsigned __int8 *)(v18 + 50) == *(_DWORD *)(a1 + 8) )
      {
        v15 += sub_31150(k[5], 1);
        v16 += sub_31150(v18, 3);
      }
    }
    if ( v15 > 0 && 100 * (v16 / v15) <= 30 && v6 <= 60 )
    {
      if ( v8 / 0xA * v41[5] >= 0x3E8 )
      {
        v19 = *(_DWORD *)(a1 + 20);
        for ( m = *(_DWORD **)(v19 + 28); m != *(_DWORD **)(v19 + 32); ++m )
        {
          if ( *m == 0 )
          {
            *m = 3;
            break;
          }
        }
        sub_16570((unsigned int *)(a1 + 144), 0x3E8u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        v21 = sub_51C14(a1, v42);
        sub_1654C((unsigned int *)(a1 + 144), v21);
      }
      goto LABEL_55;
    }
  }
  if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 4) != 0 )
  {
    v22 = *(_DWORD *)(a1 + 20);
    if ( v6 <= 5 * *(unsigned __int8 *)(v22 + 49) )
    {
      if ( v8 / 0xA * v41[5] >= 0x320 )
      {
        for ( n = *(_DWORD **)(v22 + 28); n != *(_DWORD **)(v22 + 32); ++n )
        {
          if ( *n == 0 )
          {
            *n = 4;
            break;
          }
        }
        sub_16570((unsigned int *)(a1 + 144), 0x320u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        v24 = sub_51C14(a1, v42);
        sub_1654C((unsigned int *)(a1 + 144), v24);
      }
      goto LABEL_55;
    }
  }
  if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 5) == 0 )
    goto LABEL_67;
  v25 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  v26 = 0;
  for ( ii = (_DWORD *)*sub_1DDF0((int)varg_r2a, &v40);
        ii != (_DWORD *)*sub_1DE08((void **)varg_r2a, &v40);
        ii = sub_16A30(ii) )
  {
    v28 = ii[5];
    if ( *(unsigned __int8 *)(v28 + 50) == *(_DWORD *)(a1 + 8) )
    {
      v25 += *(_WORD *)(v28 + 22);
      v26 += sub_31150(ii[5], 5);
    }
  }
  if ( v25 >= 0x32u && v6 <= 10 * (3 - v26) )
  {
    v29 = v8 / 0xA;
    v30 = v8 / 0xA * v41[5];
    if ( v30 >= 0x1388 )
    {
      sub_4A7DC(*(_DWORD *)(a1 + 20), 5);
      v31 = (unsigned int *)(a1 + 144);
      sub_16570((unsigned int *)(a1 + 144), 0x1388u);
      sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
LABEL_89:
      v37 = sub_51C14(a1, v42);
      sub_1654C(v31, v37);
    }
  }
  else
  {
LABEL_67:
    if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 6) != 0
      && v6 <= 10 * *(unsigned __int8 *)(*(_DWORD *)(a1 + 20) + 49) )
    {
      v29 = v8 / 0xA;
      v30 = v8 / 0xA * v41[5];
      if ( v30 >= 0x4B0 )
      {
        sub_4A7DC(*(_DWORD *)(a1 + 20), 6);
        v31 = (unsigned int *)(a1 + 144);
        sub_16570((unsigned int *)(a1 + 144), 0x4B0u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        goto LABEL_89;
      }
    }
    else if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 7) != 0 && v6 <= 10 )
    {
      v29 = v8 / 0xA;
      v30 = v8 / 0xA * v41[4];
      if ( v30 >= 0x7D0 )
      {
        sub_4A7DC(*(_DWORD *)(a1 + 20), 7);
        v31 = (unsigned int *)(a1 + 144);
        sub_16570((unsigned int *)(a1 + 144), 0x7D0u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        goto LABEL_89;
      }
    }
    else
    {
      if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 8) == 0 || v6 > 20 )
      {
        if ( (unsigned __int8)sub_17A90(*(_DWORD *)(a1 + 20), 7) == 0 )
          goto LABEL_90;
        v32 = 0;
        v33 = 0;
        for ( jj = (_DWORD *)*sub_2F628((int)varg_r2a, &v40);
              jj != (_DWORD *)*sub_2F640((void **)varg_r2a, v41);
              jj = sub_16A30(jj) )
        {
          v35 = jj[5];
          v36 = *(_WORD *)(v35 + 22);
          if ( *(unsigned __int8 *)(v35 + 50) == *(_DWORD *)(a1 + 8) )
            v32 += v36;
          else
            v33 = (unsigned __int16)(v33 + v36);
        }
        v29 = v33 << 16;
        v30 = v32;
        if ( v32 >= (unsigned int)((unsigned __int16)v33 >> 2) )
          goto LABEL_90;
        if ( v6 > 30 )
          goto LABEL_90;
        v29 = v8 / 0xA;
        v30 = v8 / 0xA * v41[5];
        if ( v30 < 0x2710 )
          goto LABEL_90;
        sub_4A7DC(*(_DWORD *)(a1 + 20), 9);
        v31 = (unsigned int *)(a1 + 144);
        sub_16570((unsigned int *)(a1 + 144), 0x2710u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        goto LABEL_89;
      }
      v29 = v8 / 0xA;
      v30 = v8 / 0xA * v41[4];
      if ( v30 >= 0x1388 )
      {
        sub_4A7DC(*(_DWORD *)(a1 + 20), 8);
        v31 = (unsigned int *)(a1 + 144);
        sub_16570((unsigned int *)(a1 + 144), 0x1388u);
        sub_305F0((unsigned __int16 *)(a1 + 136), 1u, &v39, &v38, (int)v42);
        goto LABEL_89;
      }
    }
  }
LABEL_90:
  CFrameWnd::OnSysCommand((CFrameWnd *)v42, v30, v29);
LABEL_91:
  sub_327B0((void **)varg_r2a);
}

