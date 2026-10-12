/* ea:   0x000198CC */
/* name: sub_198CC */
/* size: 68 */
void __fastcall sub_198CC(int a1, _DWORD *a2)
{
  _DWORD *v3; // r6
  _DWORD *v4; // r4

  v3 = a2;
  if ( a2 != nullptr )
  {
    do
    {
      sub_198CC(a1, v3[3]);
      v4 = (_DWORD *)v3[2];
      sub_19910(v3 + 4);
      free(v3);
      v3 = v4;
    }
    while ( v4 != nullptr );
  }
}

