/* ea:   0x00053714 */
/* name: sub_53714 */
/* size: 228 */
void __fastcall sub_53714(_DWORD *a1)
{
  int i; // r7
  int v3; // r6
  int v4; // r5
  int v5; // r1
  int v6; // r3
  void (__fastcall ***v7)(_DWORD, int); // r2
  void *v8; // r0

  for ( i = 0; i < *a1; ++i )
  {
    v3 = 0;
    if ( (int)a1[1] > 0 )
    {
      v4 = 4 * i;
      do
      {
        v5 = *(_DWORD *)(v4 + a1[2]);
        if ( *(_DWORD *)(v5 + 4 * v3) != 0 )
        {
          sub_11460(*(_DWORD **)(v5 + 4 * v3));
          v6 = a1[2];
          v7 = *(void (__fastcall ****)(_DWORD, int))(*(_DWORD *)(v4 + v6) + 4 * v3);
          if ( v7 != nullptr )
            (**v7)(*(_DWORD *)(*(_DWORD *)(v4 + v6) + 4 * v3), 1);
          *(_DWORD *)(*(_DWORD *)(v4 + a1[2]) + 4 * v3) = 0;
        }
        ++v3;
      }
      while ( v3 < a1[1] );
    }
    if ( *(_DWORD *)(4 * i + a1[2]) != 0 )
    {
      operator delete(*(void **)(4 * i + a1[2]));
      *(_DWORD *)(4 * i + a1[2]) = 0;
    }
  }
  v8 = (void *)a1[2];
  if ( v8 != nullptr )
  {
    operator delete(v8);
    a1[2] = 0;
  }
}

