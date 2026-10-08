/* ea:   0x000643EC */
/* name: sub_643EC */
/* size: 20 */
void __fastcall sub_643EC(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

