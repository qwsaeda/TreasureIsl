/* ea:   0x00019994 */
/* name: sub_19994 */
/* size: 68 */
void __fastcall sub_19994(int a1, _DWORD *a2)
{
  _DWORD *v3; // r6
  _DWORD *v4; // r4

  v3 = a2;
  if ( a2 != nullptr )
  {
    do
    {
      sub_19994(a1, v3[3]);
      v4 = (_DWORD *)v3[2];
      sub_19A98(v3 + 4);
      free(v3);
      v3 = v4;
    }
    while ( v4 != nullptr );
  }
}

