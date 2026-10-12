/* ea:   0x0004A7DC */
/* name: sub_4A7DC */
/* size: 44 */
int __fastcall sub_4A7DC(int result, int a2)
{
  int *v3; // r2
  int *v4; // r1

  v3 = *(int **)(result + 28);
  v4 = *(int **)(result + 32);
  while ( v3 != v4 )
  {
    result = *v3;
    if ( *v3 == 0 )
    {
      *v3 = a2;
      return result;
    }
    ++v3;
  }
  return result;
}

